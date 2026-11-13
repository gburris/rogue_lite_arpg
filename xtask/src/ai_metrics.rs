use std::collections::{BTreeMap, HashSet};
use std::f64::consts::TAU;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{fs, thread};

use anyhow::{Context, Result, ensure};
use clap::{Args, ValueEnum};
use serde_json::{Value, json};

use crate::remote::{Point, ScriptedGame, attack_input, distance, length, list, nearest, point};
use crate::sweep::enter_repository_root;
use crate::table::Table;

/// An open room with no enemies of its own, the player starting in the middle.
const ROOM_RECIPE: &str = "AiTestRoom";
const SPAWN_RADIUS: f64 = 220.0;
/// Player health during a scenario, topped up whenever it falls below half, so hits register
/// without the player dying.
const SCENARIO_HEALTH: f64 = 100_000.0;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(33);
/// Intended speed below which an enemy counts as standing still.
const MOVING_SPEED: f64 = 20.0;
/// Fraction of its intended travel an enemy must cover to not count as stuck.
const STUCK_TRAVEL_FRACTION: f64 = 0.25;
/// Cosine between successive intended velocities below which a turn counts as a reversal.
const REVERSAL_COSINE: f64 = -0.3;
/// Seconds between the strafing player's direction changes.
const STRAFE_PERIOD: f64 = 1.5;
/// Distance at which the fighting player stops closing in and just swings.
const FIGHT_RANGE: f64 = 70.0;

/// Column names of the saved table, after `kind` and `script`. Rates are per enemy-minute alive;
/// `actions` counts each start of an equipment use or ability.
/// `notice_s` is the median time from spawn to first targeting the player, counting an enemy that
/// never does as the whole scenario.
const METRICS: [&str; 10] = [
    "notice_s",
    "engaged_%",
    "stuck_s/min",
    "reversals/min",
    "switches/min",
    "actions/min",
    "hits/min",
    "damage/min",
    "distance",
    "killed",
];

/// Options for `ai-metrics`.
#[derive(Args)]
pub(super) struct AiMetricsArgs {
    #[arg(long, default_value_t = 20.0, help = "Game seconds per scenario")]
    seconds: f64,

    #[arg(long, default_value_t = 3, help = "Enemies spawned per scenario")]
    enemies: usize,

    #[arg(
        long,
        default_value_t = 3,
        value_parser = clap::value_parser!(u32).range(1..=20),
        help = "Runs per scenario; the table shows each metric's median"
    )]
    repeats: u32,

    #[arg(
        long,
        value_delimiter = ',',
        default_values = ["Warrior", "IceMage", "FireMage", "Bat"],
        help = "Enemy kinds, one scenario set each"
    )]
    kinds: Vec<String>,

    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = [PlayerScript::Still, PlayerScript::Strafe, PlayerScript::Fight],
        help = "What the player does in each scenario"
    )]
    scripts: Vec<PlayerScript>,

    #[arg(
        long,
        help = "Metrics file to compare against; defaults to the newest other one in the folder"
    )]
    baseline: Option<PathBuf>,

    #[arg(
        long,
        default_value = "runs/ai_metrics",
        help = "Where `<commit>.tsv` and the per-scenario recordings go"
    )]
    output_directory: PathBuf,
}

/// What the player does while enemies engage.
#[derive(ValueEnum, Clone, Copy, Debug)]
enum PlayerScript {
    Still,
    Strafe,
    Fight,
}

/// One scenario's metrics, in [`METRICS`] order, each the median over the repeats.
struct ScenarioMetrics {
    kind: String,
    script: PlayerScript,
    values: Vec<f64>,
}

/// Plays each enemy kind against each player script in the AI arena, measures how the enemies
/// moved and fought, saves the table as `<commit>.tsv` and compares it with a baseline.
pub(super) fn run(args: &AiMetricsArgs) -> Result<()> {
    enter_repository_root()?;
    let commit = commit_name()?;
    let recordings = args.output_directory.join(&commit);
    fs::create_dir_all(&recordings)
        .with_context(|| format!("creating {}", recordings.display()))?;

    let mut game = ScriptedGame::launch(&recordings.join("game.log"))?;
    game.call("game/start", Value::Null)?;
    game.wait_until_playing(Duration::from_secs(120))?;
    thread::sleep(Duration::from_secs(1));
    game.equip_from_inventory(&["Sword"])?;

    let mut results = Vec::new();
    for kind in &args.kinds {
        for &script in &args.scripts {
            let name = format!("{kind}_{}", script.name());
            println!("== {name}");
            let mut repeats = Vec::new();
            for repeat in 1..=args.repeats {
                let frames = record_scenario(&mut game, kind, script, args)?;
                let file = recordings.join(format!("{name}_r{repeat}.jsonl"));
                save_recording(&frames, &file)?;
                repeats.push(measure(&frames, args.seconds));
            }
            results.push(ScenarioMetrics {
                kind: kind.clone(),
                script,
                values: medians(&repeats),
            });
        }
    }
    game.exit()?;

    let metrics_file = args.output_directory.join(format!("{commit}.tsv"));
    fs::write(&metrics_file, to_tsv(&results))
        .with_context(|| format!("writing {}", metrics_file.display()))?;
    let baseline_file = match &args.baseline {
        Some(file) => Some(file.clone()),
        None => newest_other_metrics_file(&args.output_directory, &metrics_file)?,
    };
    let baseline = match &baseline_file {
        Some(file) => read_tsv(file)?,
        None => BTreeMap::new(),
    };

    println!("\n{}", comparison_table(&results, &baseline));
    println!("saved {}", metrics_file.display());
    if let Some(file) = baseline_file {
        println!("(changes) are against {}", file.display());
    }
    Ok(())
}

/// Loads the test room, spawns a ring of `kind` around the player and samples snapshots for
/// `args.seconds` of game time while the player follows `script`. Each frame keeps only the
/// spawned enemies.
fn record_scenario(
    game: &mut ScriptedGame,
    kind: &str,
    script: PlayerScript,
    args: &AiMetricsArgs,
) -> Result<Vec<Value>> {
    game.release_player_input()?;
    game.load_map(ROOM_RECIPE, 1)?;
    thread::sleep(Duration::from_millis(500));
    game.set_player_health(SCENARIO_HEALTH, SCENARIO_HEALTH)?;

    // Hits from before the scenario don't count.
    let snapshot = game.snapshot_with_new_hits()?;
    let center = point(&snapshot["player"]["position"]).context("player has no position")?;
    let positions: Vec<Point> = (0..args.enemies)
        .map(|index| {
            let angle = TAU * index as f64 / args.enemies as f64;
            [
                center[0] + SPAWN_RADIUS * angle.cos(),
                center[1] + SPAWN_RADIUS * angle.sin(),
            ]
        })
        .collect();
    let spawned: HashSet<u64> = game.spawn_enemies(kind, &positions)?.into_iter().collect();

    let started_at = snapshot["time"].as_f64().unwrap_or_default();
    let mut frames = Vec::new();
    loop {
        let mut snapshot = game.snapshot_with_new_hits()?;
        let elapsed = snapshot["time"].as_f64().unwrap_or_default() - started_at;
        if elapsed >= args.seconds {
            break;
        }
        if snapshot["player"]["hp"].as_f64().unwrap_or(SCENARIO_HEALTH) < SCENARIO_HEALTH / 2.0 {
            game.set_player_health(SCENARIO_HEALTH, SCENARIO_HEALTH)?;
        }
        if let Some(enemies) = snapshot["enemies"].as_array_mut() {
            enemies.retain(|enemy| {
                enemy["entity"]
                    .as_u64()
                    .is_some_and(|entity| spawned.contains(&entity))
            });
        }
        game.call(
            "game/player_input",
            player_input(script, &snapshot, elapsed),
        )?;
        frames.push(snapshot);
        thread::sleep(SAMPLE_INTERVAL);
    }
    game.release_player_input()?;
    Ok(frames)
}

fn player_input(script: PlayerScript, snapshot: &Value, elapsed: f64) -> Value {
    match script {
        PlayerScript::Still => json!({}),
        PlayerScript::Strafe => {
            let direction = if (elapsed / STRAFE_PERIOD) as u64 % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            json!({ "move": [direction, 0.0] })
        }
        PlayerScript::Fight => {
            let Some(position) = point(&snapshot["player"]["position"]) else {
                return json!({});
            };
            let enemy_positions =
                list(&snapshot["enemies"]).filter_map(|enemy| point(&enemy["position"]));

            match nearest(position, enemy_positions) {
                Some(enemy) => attack_input(position, enemy, FIGHT_RANGE),
                None => json!({}),
            }
        }
    }
}

/// Totals over every spawned enemy, from which [`measure`] takes per-minute rates.
#[derive(Default)]
struct Totals {
    alive_seconds: f64,
    engaged_seconds: f64,
    stuck_seconds: f64,
    reversals: u32,
    switches: u32,
    actions: u32,
    target_distance_sum: f64,
    target_distance_samples: u32,
}

/// The [`METRICS`] of one recording of a `seconds`-long scenario.
fn measure(frames: &[Value], seconds: f64) -> Vec<f64> {
    let mut totals = Totals::default();
    let mut timelines: BTreeMap<u64, Vec<(f64, &Value)>> = BTreeMap::new();
    for frame in frames {
        let time = frame["time"].as_f64().unwrap_or_default();
        for enemy in list(&frame["enemies"]) {
            if let Some(entity) = enemy["entity"].as_u64() {
                timelines.entry(entity).or_default().push((time, enemy));
            }
        }
    }
    for timeline in timelines.values() {
        add_enemy_totals(timeline, &mut totals);
    }
    let first_time = frames
        .first()
        .map_or(0.0, |frame| frame["time"].as_f64().unwrap_or_default());
    let notice_times: Vec<f64> = timelines
        .values()
        .map(|timeline| {
            timeline
                .iter()
                .find(|(_, enemy)| enemy["has_target"] == true)
                .map_or(seconds, |(time, _)| time - first_time)
        })
        .collect();

    let player = frames.last().map(|frame| frame["player"]["entity"].clone());
    let mut hits = 0;
    let mut damage = 0.0;
    for hit in frames.iter().flat_map(|frame| list(&frame["hits"])) {
        let attacker = hit["attacker"].as_u64();
        if Some(&hit["entity"]) == player.as_ref()
            && attacker.is_some_and(|attacker| timelines.contains_key(&attacker))
        {
            hits += 1;
            damage += hit["damage"].as_f64().unwrap_or_default();
        }
    }
    let last_time = frames
        .last()
        .map(|frame| frame["time"].as_f64().unwrap_or_default());
    let killed = timelines
        .values()
        .filter(|timeline| {
            let (time, enemy) = timeline[timeline.len() - 1];
            Some(time) != last_time || enemy["hp"].as_f64().unwrap_or_default() <= 0.0
        })
        .count();

    let minutes = totals.alive_seconds / 60.0;
    let per_minute = |count: f64| if minutes > 0.0 { count / minutes } else { 0.0 };
    vec![
        median(notice_times).unwrap_or(seconds),
        if totals.alive_seconds > 0.0 {
            100.0 * totals.engaged_seconds / totals.alive_seconds
        } else {
            0.0
        },
        per_minute(totals.stuck_seconds),
        per_minute(f64::from(totals.reversals)),
        per_minute(f64::from(totals.switches)),
        per_minute(f64::from(totals.actions)),
        per_minute(f64::from(hits)),
        per_minute(damage),
        if totals.target_distance_samples > 0 {
            totals.target_distance_sum / f64::from(totals.target_distance_samples)
        } else {
            0.0
        },
        killed as f64,
    ]
}

fn add_enemy_totals(timeline: &[(f64, &Value)], totals: &mut Totals) {
    let mut last_moving_velocity: Option<Point> = None;
    for pair in timeline.windows(2) {
        let [(previous_time, previous), (time, enemy)] = pair else {
            continue;
        };
        // A dead enemy stays in the snapshot with its last intended velocity.
        if enemy["hp"].as_f64().unwrap_or_default() <= 0.0 {
            break;
        }
        let seconds = time - previous_time;
        totals.alive_seconds += seconds;
        if enemy["has_target"] == true {
            totals.engaged_seconds += seconds;
            if let Some(distance) = enemy["target_distance"].as_f64() {
                totals.target_distance_sum += distance;
                totals.target_distance_samples += 1;
            }
        }
        if enemy["motion"] != previous["motion"] {
            totals.switches += 1;
        }
        let is_action = |action: &Value| action.as_str().is_some_and(|action| action != "Idle");
        if is_action(&enemy["action"]) && enemy["action"] != previous["action"] {
            totals.actions += 1;
        }

        let intended = point(&enemy["intended_velocity"]).unwrap_or_default();
        let speed = length(intended);
        if speed <= MOVING_SPEED {
            continue;
        }
        let moved = match (point(&previous["position"]), point(&enemy["position"])) {
            (Some(from), Some(to)) => distance(from, to),
            _ => 0.0,
        };
        if seconds > 0.0
            && enemy["stunned"] != true
            && moved < STUCK_TRAVEL_FRACTION * speed * seconds
        {
            totals.stuck_seconds += seconds;
        }
        if let Some(last) = last_moving_velocity {
            let cosine = (intended[0] * last[0] + intended[1] * last[1]) / (speed * length(last));
            if cosine < REVERSAL_COSINE {
                totals.reversals += 1;
            }
        }
        last_moving_velocity = Some(intended);
    }
}

/// Each metric's median over the repeats' metric lists.
fn medians(repeats: &[Vec<f64>]) -> Vec<f64> {
    (0..METRICS.len())
        .map(|metric| {
            median(repeats.iter().map(|values| values[metric]).collect()).unwrap_or_default()
        })
        .collect()
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Some(if values.len() % 2 == 0 {
        f64::midpoint(values[middle - 1], values[middle])
    } else {
        values[middle]
    })
}

fn save_recording(frames: &[Value], path: &Path) -> Result<()> {
    let mut text = String::new();
    for frame in frames {
        let _ = writeln!(text, "{frame}");
    }
    fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

/// The short hash of `HEAD`, with `-dirty` when tracked files have uncommitted changes.
fn commit_name() -> Result<String> {
    let git = |args: &[&str]| -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .output()
            .context("running git")?;
        ensure!(output.status.success(), "git {args:?} failed");
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    let hash = git(&["rev-parse", "--short", "HEAD"])?;
    let dirty = !git(&["status", "--porcelain", "--untracked-files=no"])?.is_empty();
    Ok(if dirty { format!("{hash}-dirty") } else { hash })
}

fn newest_other_metrics_file(directory: &Path, current: &Path) -> Result<Option<PathBuf>> {
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "tsv") || path == current {
            continue;
        }
        let modified = path.metadata()?.modified()?;
        if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
            newest = Some((modified, path));
        }
    }
    Ok(newest.map(|(_, path)| path))
}

fn to_tsv(results: &[ScenarioMetrics]) -> String {
    let mut text = format!("kind\tscript\t{}\n", METRICS.join("\t"));
    for result in results {
        let values: Vec<String> = result
            .values
            .iter()
            .map(|value| format!("{value:.2}"))
            .collect();
        let _ = writeln!(
            text,
            "{}\t{}\t{}",
            result.kind,
            result.script.name(),
            values.join("\t")
        );
    }
    text
}

/// A saved table, keyed by `(kind, script)`, each row's values by metric name.
type SavedMetrics = BTreeMap<(String, String), BTreeMap<String, f64>>;

fn read_tsv(path: &Path) -> Result<SavedMetrics> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let mut saved = SavedMetrics::new();
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let [kind, script, values @ ..] = cells.as_slice() else {
            continue;
        };
        let values = header
            .iter()
            .skip(2)
            .zip(values)
            .filter_map(|(name, value)| Some(((*name).to_owned(), value.parse().ok()?)))
            .collect();
        saved.insert(((*kind).to_owned(), (*script).to_owned()), values);
    }
    Ok(saved)
}

/// One row per scenario; each cell is the new value with its change from the baseline.
fn comparison_table(results: &[ScenarioMetrics], baseline: &SavedMetrics) -> Table {
    let mut table = Table::new(["kind", "script"].into_iter().chain(METRICS));
    for result in results {
        let before = baseline.get(&(result.kind.clone(), result.script.name().to_owned()));
        let mut cells = vec![result.kind.clone(), result.script.name().to_owned()];
        for (name, value) in METRICS.iter().zip(&result.values) {
            let cell = match before.and_then(|before| before.get(*name)) {
                Some(before) => format!("{value:.1} ({:+.1})", value - before),
                None => format!("{value:.1}"),
            };
            cells.push(cell);
        }
        table.add_row(cells);
    }
    table
}

impl PlayerScript {
    fn name(self) -> &'static str {
        match self {
            Self::Still => "still",
            Self::Strafe => "strafe",
            Self::Fight => "fight",
        }
    }
}
