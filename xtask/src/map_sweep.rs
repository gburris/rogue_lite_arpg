use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

use anyhow::{Context, Result};
use clap::Args;
use serde_json::{Value, json};

use crate::remote::{RemoteArgs, ScriptedGame, distance, length, list, point};
use crate::sweep::enter_repository_root;
use crate::table::Table;

/// Real seconds between the two snapshots that decide whether an enemy is stuck.
const SETTLE_TIME: Duration = Duration::from_secs(2);
/// Intended speed above which an enemy is trying to move.
const MOVING_SPEED: f64 = 20.0;
/// Distance an enemy trying to move must cover over [`SETTLE_TIME`] to not count as stuck.
const STUCK_DISTANCE: f64 = 5.0;

/// Options for `map-sweep`.
#[derive(Args)]
pub(super) struct MapSweepArgs {
    #[command(flatten)]
    remote: RemoteArgs,

    #[arg(
        long,
        value_delimiter = ',',
        default_values = [
            "Swamp",
            "SwampWithALotOfEmptySquares",
            "LongHallway",
            "TreasureRoom",
            "FireShrine",
            "SkyTempleAscent",
        ],
        help = "Map recipes to build"
    )]
    recipes: Vec<String>,

    #[arg(long, default_value_t = 10, help = "Seeds per recipe, starting at 1")]
    seeds: u64,

    #[arg(long, help = "Screenshot every map, not only flagged ones")]
    screenshots: bool,

    #[arg(
        long,
        default_value = "runs/map_sweep",
        help = "Where `sweep.tsv`, flagged screenshots and the game log go"
    )]
    output_directory: PathBuf,
}

/// What one built map held.
struct MapReport {
    recipe: String,
    seed: u64,
    enemies: usize,
    chests: usize,
    portals: usize,
    nearest_enemy: Option<f64>,
    stuck: Vec<String>,
}

/// Builds each recipe with each seed and flags maps with enemies that try to move but stay put,
/// which usually means they spawned inside or against a wall. Saves a row per map to
/// `sweep.tsv`, and a screenshot of each flagged map.
pub(super) fn run(args: &MapSweepArgs) -> Result<()> {
    enter_repository_root()?;
    fs::create_dir_all(&args.output_directory)
        .with_context(|| format!("creating {}", args.output_directory.display()))?;

    let mut game = ScriptedGame::launch(&args.output_directory.join("game.log"), args.remote)?;
    game.call("game/start", Value::Null)?;
    game.wait_until_playing(Duration::from_secs(120))?;

    let mut reports = Vec::new();
    for recipe in &args.recipes {
        for seed in 1..=args.seeds {
            let report = inspect_map(&mut game, recipe, seed)?;
            if !report.stuck.is_empty() || args.screenshots {
                let file = args.output_directory.join(format!("{recipe}_{seed}.png"));
                game.screenshot(&file)?;
            }
            println!(
                "{recipe} seed {seed}: {} enemies, {} stuck",
                report.enemies,
                report.stuck.len()
            );
            reports.push(report);
        }
    }
    game.exit()?;

    let tsv_file = args.output_directory.join("sweep.tsv");
    fs::write(&tsv_file, to_tsv(&reports))
        .with_context(|| format!("writing {}", tsv_file.display()))?;
    println!("\n{}", summary_table(&reports));
    let flagged: Vec<&MapReport> = reports.iter().filter(|r| !r.stuck.is_empty()).collect();
    for report in &flagged {
        println!(
            "flagged {} seed {}: {}",
            report.recipe,
            report.seed,
            report.stuck.join(", ")
        );
    }
    println!("saved {}", tsv_file.display());
    Ok(())
}

fn inspect_map(game: &mut ScriptedGame, recipe: &str, seed: u64) -> Result<MapReport> {
    game.load_map(recipe, seed)?;
    game.call("game/set_invincible", json!({ "invincible": true }))?;
    thread::sleep(Duration::from_millis(500));

    let before = game.snapshot()?;
    thread::sleep(SETTLE_TIME);
    let after = game.snapshot()?;

    let start = point(&before["player"]["position"]).unwrap_or_default();
    let positions_before: HashMap<u64, [f64; 2]> = enemies(&before)
        .filter_map(|enemy| Some((enemy["entity"].as_u64()?, point(&enemy["position"])?)))
        .collect();
    let stuck = enemies(&after)
        .filter_map(|enemy| {
            let from = positions_before.get(&enemy["entity"].as_u64()?)?;
            let to = point(&enemy["position"])?;
            let intended_speed = length(point(&enemy["intended_velocity"])?);
            (intended_speed > MOVING_SPEED && distance(*from, to) < STUCK_DISTANCE).then(|| {
                format!(
                    "{} {} at ({:.0}, {:.0})",
                    enemy["kind"].as_str().unwrap_or("?"),
                    enemy["motion"].as_str().unwrap_or("-"),
                    to[0],
                    to[1]
                )
            })
        })
        .collect();
    let nearest_enemy = enemies(&before)
        .filter_map(|enemy| point(&enemy["position"]))
        .map(|position| distance(start, position))
        .min_by(f64::total_cmp);
    let chests = list(&before["interactables"])
        .filter(|interactable| interactable["name"] == "Chest")
        .count();

    Ok(MapReport {
        recipe: recipe.to_owned(),
        seed,
        enemies: enemies(&before).count(),
        chests,
        portals: list(&before["portals"]).count(),
        nearest_enemy,
        stuck,
    })
}

fn enemies(snapshot: &Value) -> impl Iterator<Item = &Value> {
    list(&snapshot["enemies"])
}

fn to_tsv(reports: &[MapReport]) -> String {
    let mut text = "recipe\tseed\tenemies\tchests\tportals\tnearest_enemy\tstuck\n".to_owned();
    for report in reports {
        let _ = writeln!(
            text,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            report.recipe,
            report.seed,
            report.enemies,
            report.chests,
            report.portals,
            report
                .nearest_enemy
                .map_or_else(String::new, |nearest| format!("{nearest:.0}")),
            report.stuck.join("; ")
        );
    }
    text
}

/// One row per recipe: how many maps it built, enemy and chest ranges, and flagged maps.
fn summary_table(reports: &[MapReport]) -> Table {
    let mut table = Table::new([
        "recipe",
        "maps",
        "enemies",
        "chests",
        "closest enemy",
        "flagged",
    ]);
    let mut recipes: Vec<&str> = reports
        .iter()
        .map(|report| report.recipe.as_str())
        .collect();
    recipes.dedup();
    for recipe in recipes {
        let maps: Vec<&MapReport> = reports.iter().filter(|r| r.recipe == recipe).collect();
        let range = |values: Vec<usize>| {
            let low = values.iter().min().copied().unwrap_or_default();
            let high = values.iter().max().copied().unwrap_or_default();
            format!("{low}-{high}")
        };
        let closest = maps
            .iter()
            .filter_map(|report| report.nearest_enemy)
            .min_by(f64::total_cmp)
            .map_or_else(|| "-".to_owned(), |closest| format!("{closest:.0}"));
        table.add_row(vec![
            recipe.to_owned(),
            maps.len().to_string(),
            range(maps.iter().map(|report| report.enemies).collect()),
            range(maps.iter().map(|report| report.chests).collect()),
            closest,
            maps.iter()
                .filter(|report| !report.stuck.is_empty())
                .count()
                .to_string(),
        ]);
    }
    table
}
