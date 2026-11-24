use std::collections::{BTreeMap, HashSet};
use std::f64::consts::TAU;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{fs, thread};

use anyhow::{Context, Result, ensure};
use clap::{Args, ValueEnum};
use serde_json::{Value, json};

use crate::behavior_coverage::{
    BehaviorCoverage, coverage_table, find_behaviors_no_longer_started, to_coverage_rates,
};
use crate::metrics_comparison::{
    BASELINE_REPEATS_FOR_BAND, Baseline, CELLS_OUTSIDE_BAND_TO_FAIL, comparison_table,
    find_cells_outside_band, newest_other_metrics_file, read_baseline, save_tables,
};
use crate::remote::{
    Point, RemoteArgs, ScriptedGame, attack_input, distance, length, list, nearest, point,
    report_log_problems,
};
use crate::sweep::enter_repository_root;

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
pub(super) const METRICS: [&str; 10] = [
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
    #[command(flatten)]
    remote: RemoteArgs,

    #[arg(long, default_value_t = 20.0, help = "Game seconds per scenario")]
    seconds: f64,

    #[arg(
        long,
        default_value_t = 2.0,
        help = "Game speed during scenarios; above 1 the run is shorter but less like real play"
    )]
    speed: f64,

    #[arg(
        long,
        default_value = "AiTestRoom",
        help = "Map recipe each scenario loads: `AiTestRoom`, or `AiTestRoomWall` with a wall"
    )]
    room: String,

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
        default_value_t = 4,
        value_parser = clap::value_parser!(u32).range(1..=8),
        help = "Most games played at once, on ports counting up from --remote-port; the kinds are \
                split evenly between them, which can take fewer games"
    )]
    games: u32,

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
        help = "Metrics file to compare against, with the `repeats.tsv` and `behaviors.tsv` in \
                the folder of its name; defaults to the newest other one in the folder"
    )]
    baseline: Option<PathBuf>,

    #[arg(
        long,
        help = "Measure a saved recordings folder, such as `runs/ai_metrics/<commit>`, instead of \
                playing, saving its tables beside and inside it; --seconds must match the run \
                that recorded it"
    )]
    rebuild_from: Option<PathBuf>,

    #[arg(
        long,
        default_value = "runs/ai_metrics",
        help = "Where `<commit>.tsv`, `<commit>/repeats.tsv`, `<commit>/behaviors.tsv` and the \
                per-scenario recordings go"
    )]
    output_directory: PathBuf,
}

/// What the player does while enemies engage.
#[derive(ValueEnum, Clone, Copy, Debug)]
pub(super) enum PlayerScript {
    Still,
    Strafe,
    Fight,
}

/// One scenario's metrics for each repeat, in [`METRICS`] order.
pub(super) struct ScenarioMetrics {
    pub(super) kind: String,
    pub(super) script: PlayerScript,
    pub(super) repeats: Vec<Vec<f64>>,
}

impl ScenarioMetrics {
    /// The `(kind, script)` the saved tables key this scenario by.
    pub(super) fn key(&self) -> (String, String) {
        (self.kind.clone(), self.script.name().to_owned())
    }

    /// Each metric's median over the repeats.
    pub(super) fn medians(&self) -> Vec<f64> {
        (0..METRICS.len())
            .map(|metric| {
                median(self.repeats.iter().map(|values| values[metric]).collect())
                    .unwrap_or_default()
            })
            .collect()
    }
}

/// Plays each enemy kind against each player script in the AI arena (or reads a saved run's
/// recordings), measures how the enemies moved and fought, and saves the tables (see
/// [`save_tables`]). Compares them with a baseline, and fails on a game log problem, on
/// [`CELLS_OUTSIDE_BAND_TO_FAIL`] metrics outside noise bands of [`BASELINE_REPEATS_FOR_BAND`]
/// repeats, or on a behavior that no longer starts.
pub(super) fn run(args: &AiMetricsArgs) -> Result<()> {
    enter_repository_root()?;
    let recordings = if let Some(folder) = &args.rebuild_from {
        folder.clone()
    } else {
        let recordings = args.output_directory.join(commit_name()?);
        play_scenarios(args, &recordings)?;
        recordings
    };
    let (scenarios, coverage) = measure_recordings(args, &recordings)?;
    let coverage_rates = to_coverage_rates(&coverage);
    let metrics_file = save_tables(&recordings, &scenarios, &coverage_rates)?;

    let baseline_file = match &args.baseline {
        Some(file) => Some(file.clone()),
        None => newest_other_metrics_file(&metrics_file)?,
    };
    let baseline = match &baseline_file {
        Some(file) => read_baseline(file)?,
        None => Baseline::default(),
    };
    let cells_outside_band = find_cells_outside_band(&scenarios, &baseline);
    let no_longer_started = find_behaviors_no_longer_started(
        &coverage,
        |kind| baseline.played_same_scripts(&scenarios, kind),
        &coverage_rates,
        &baseline.coverage,
    );

    println!("\n{}", comparison_table(&scenarios, &baseline));
    println!(
        "{}",
        coverage_table(&coverage, &coverage_rates, &baseline.coverage)
    );
    println!("saved {}", metrics_file.display());
    if let Some(file) = &baseline_file {
        println!("(changes) are against {}", file.display());
        let without_full_band = baseline.count_scenarios_without_full_band(&scenarios);
        if without_full_band > 0 {
            println!(
                "band: {without_full_band} scenarios have no baseline band of \
                 {BASELINE_REPEATS_FOR_BAND} repeats, so their cells don't fail the run"
            );
        }
    }
    for cell in &cells_outside_band {
        println!("band: {cell}");
    }
    for (kind, behavior) in &no_longer_started {
        println!("coverage: {kind} {behavior} started in the baseline but never starts now");
    }
    report_log_problems(&find_log_files(&recordings)?)?;
    let failing_cells = cells_outside_band
        .iter()
        .filter(|cell| cell.counts_toward_failure())
        .count();
    ensure!(
        failing_cells < CELLS_OUTSIDE_BAND_TO_FAIL,
        "{failing_cells} metrics left the baseline's noise band"
    );
    ensure!(
        no_longer_started.is_empty(),
        "{} behaviors no longer start",
        no_longer_started.len()
    );
    Ok(())
}

/// Plays every scenario, each enemy kind in one of several games at once, saving each repeat's
/// recording and each game's log to `recordings`, emptied first so an earlier run's files don't
/// count.
fn play_scenarios(args: &AiMetricsArgs, recordings: &Path) -> Result<()> {
    if recordings.exists() {
        fs::remove_dir_all(recordings)
            .with_context(|| format!("clearing {}", recordings.display()))?;
    }
    fs::create_dir_all(recordings).with_context(|| format!("creating {}", recordings.display()))?;
    let kinds_per_game = args.kinds.len().div_ceil(args.games as usize);
    let kind_groups: Vec<&[String]> = args.kinds.chunks(kinds_per_game).collect();
    let remotes = (0..kind_groups.len())
        .map(|index| args.remote.with_port_offset(index as u16))
        .collect::<Result<Vec<_>>>()?;
    // Set when a game fails, so the others stop instead of finishing results nobody keeps.
    let game_failed = AtomicBool::new(false);
    let game_outcomes: Vec<Result<()>> = thread::scope(|scope| {
        let mut handles = Vec::new();
        for (index, (kinds, remote)) in kind_groups.iter().zip(remotes).enumerate() {
            let log_file = recordings.join(format!("game_{}.log", index + 1));
            let game_failed = &game_failed;
            handles.push(scope.spawn(move || {
                let outcome = play_kinds(args, kinds, remote, &log_file, recordings, game_failed);
                if outcome.is_err() {
                    game_failed.store(true, Ordering::Relaxed);
                }
                outcome
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("an ai-metrics game thread panicked"))
            .collect()
    });
    // A game stopped by another's failure returns `Ok`, so this is the failure that stopped it.
    game_outcomes.into_iter().collect()
}

/// Measures every saved repeat of `args.kinds` and `args.scripts` in `recordings`, returning each
/// scenario's metrics and each kind's behavior coverage.
fn measure_recordings(
    args: &AiMetricsArgs,
    recordings: &Path,
) -> Result<(Vec<ScenarioMetrics>, BTreeMap<String, BehaviorCoverage>)> {
    let mut scenarios = Vec::new();
    let mut coverage = BTreeMap::new();
    for kind in &args.kinds {
        let mut kind_coverage = BehaviorCoverage::default();
        for &script in &args.scripts {
            let mut repeats = Vec::new();
            for repeat in 1.. {
                let file = recordings.join(recording_file_name(kind, script, repeat));
                if !file.exists() {
                    break;
                }
                let frames = read_recording(&file)?;
                for timeline in enemy_timelines(&frames).values() {
                    kind_coverage.add_enemy_timeline(timeline);
                }
                repeats.push(measure(&frames, args.seconds));
            }
            if !repeats.is_empty() {
                scenarios.push(ScenarioMetrics {
                    kind: kind.clone(),
                    script,
                    repeats,
                });
            }
        }
        coverage.insert(kind.clone(), kind_coverage);
    }
    ensure!(
        !scenarios.is_empty(),
        "{} holds no recordings of the given kinds and scripts",
        recordings.display()
    );
    Ok((scenarios, coverage))
}

fn recording_file_name(kind: &str, script: PlayerScript, repeat: u32) -> String {
    format!("{kind}_{}_r{repeat}.jsonl", script.name())
}

fn find_log_files(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut log_files = Vec::new();
    for entry in fs::read_dir(folder).with_context(|| format!("reading {}", folder.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "log") {
            log_files.push(path);
        }
    }
    log_files.sort();
    Ok(log_files)
}

/// Launches a game on `remote`'s port and plays every scenario of `kinds` in it, saving each
/// recording to `recordings`. Stops early once `game_failed` is set.
fn play_kinds(
    args: &AiMetricsArgs,
    kinds: &[String],
    remote: RemoteArgs,
    log_file: &Path,
    recordings: &Path,
    game_failed: &AtomicBool,
) -> Result<()> {
    let mut game = ScriptedGame::launch(log_file, remote)?;
    game.call("game/start", Value::Null)?;
    game.wait_until_playing(Duration::from_secs(120))?;
    thread::sleep(Duration::from_secs(1));
    game.equip_from_inventory(&["Sword"])?;
    game.call("game/time", json!({ "speed": args.speed }))?;

    for kind in kinds {
        for &script in &args.scripts {
            println!("== {kind}_{}", script.name());
            for repeat in 1..=args.repeats {
                if game_failed.load(Ordering::Relaxed) {
                    return Ok(());
                }
                let frames = record_scenario(&mut game, kind, script, args)?;
                let file = recordings.join(recording_file_name(kind, script, repeat));
                save_recording(&frames, &file)?;
            }
        }
    }
    game.exit()
}

/// Loads `args.room`, spawns a ring of `kind` around the player and samples snapshots for
/// `args.seconds` of game time while the player follows `script`. Each frame keeps only the
/// spawned enemies.
fn record_scenario(
    game: &mut ScriptedGame,
    kind: &str,
    script: PlayerScript,
    args: &AiMetricsArgs,
) -> Result<Vec<Value>> {
    game.release_player_input()?;
    game.load_map(&args.room, 1)?;
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
        // Sampled at the same game-time interval whatever the speed.
        thread::sleep(SAMPLE_INTERVAL.div_f64(args.speed));
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
    let timelines = enemy_timelines(frames);
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

/// Each enemy's `(game time, snapshot entry)` samples in order, by entity.
fn enemy_timelines(frames: &[Value]) -> BTreeMap<u64, Vec<(f64, &Value)>> {
    let mut timelines: BTreeMap<u64, Vec<(f64, &Value)>> = BTreeMap::new();
    for frame in frames {
        let time = frame["time"].as_f64().unwrap_or_default();
        for enemy in list(&frame["enemies"]) {
            if let Some(entity) = enemy["entity"].as_u64() {
                timelines.entry(entity).or_default().push((time, enemy));
            }
        }
    }
    timelines
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

fn read_recording(path: &Path) -> Result<Vec<Value>> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    text.lines()
        .map(|line| {
            serde_json::from_str(line).with_context(|| format!("parsing {}", path.display()))
        })
        .collect()
}

fn save_recording(frames: &[Value], path: &Path) -> Result<()> {
    let mut text = String::new();
    for frame in frames {
        let _ = writeln!(text, "{frame}");
    }
    fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

/// The short hash of `HEAD`, with `-dirty` when tracked files have uncommitted changes.
pub(super) fn commit_name() -> Result<String> {
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

impl PlayerScript {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Still => "still",
            Self::Strafe => "strafe",
            Self::Fight => "fight",
        }
    }
}
