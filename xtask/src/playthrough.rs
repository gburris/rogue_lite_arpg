use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs, thread};

use anyhow::{Context, Result, ensure};
use clap::Args;
use serde_json::{Value, json};

use crate::remote::{
    MouseButton, POLL_INTERVAL, RemoteArgs, ScriptedGame, attack_input, distance, is_playing, list,
    log_problems, nearest, point,
};
use crate::sweep::enter_repository_root;
use crate::table::{Table, optional_cell};

/// Items the playthrough equips from the starting inventory.
const MAINHAND: &str = "Sword";
const OFFHAND: &str = "Knight Shield";
/// The item the playthrough drops and picks back up.
const DROPPED_ITEM: &str = "Axe";

const STATE_TIMEOUT: Duration = Duration::from_secs(60);
const PICKUP_TIMEOUT: Duration = Duration::from_secs(5);
/// Real time a new scene gets to finish spawning and fading in before a step looks at it.
const SCENE_SETTLE: Duration = Duration::from_secs(1);
/// Real time the pause menu stays open between the two snapshots that must match.
const PAUSE_WATCH: Duration = Duration::from_millis(500);
/// Real time an interactable gets to open its menu after Space.
const INTERACT_WAIT: Duration = Duration::from_millis(600);

const FIGHT_SECONDS: f64 = 8.0;
/// Distance at which the fighting player stops closing in and just swings.
const FIGHT_REACH: f64 = 60.0;

/// Options for `playthrough`.
#[derive(Args)]
pub(super) struct PlaythroughArgs {
    #[command(flatten)]
    remote: RemoteArgs,

    #[arg(
        long,
        default_value = "runs/playthrough",
        help = "Where the step screenshots and the game log go; a rerun overwrites them"
    )]
    output_directory: PathBuf,
}

/// One step of the playthrough: it drives the game, then returns a note on what it saw.
struct Step {
    name: &'static str,
    run: fn(&mut Playthrough) -> Result<String>,
}

const STEPS: &[Step] = &[
    Step {
        name: "start",
        run: start,
    },
    Step {
        name: "inventory",
        run: equip_items,
    },
    Step {
        name: "pause",
        run: pause,
    },
    Step {
        name: "items",
        run: use_items,
    },
    Step {
        name: "npcs",
        run: talk_to_npcs,
    },
    Step {
        name: "portal",
        run: take_portal,
    },
    Step {
        name: "fight",
        run: fight,
    },
    Step {
        name: "death",
        run: die,
    },
    Step {
        name: "restart",
        run: restart,
    },
];

/// The running game and where the current step saves its screenshot.
struct Playthrough {
    game: ScriptedGame,
    output_directory: PathBuf,
    step_index: usize,
    screenshots_taken: usize,
}

impl Playthrough {
    /// Saves a screenshot named after the current step and `label`.
    fn screenshot(&mut self, label: &str) -> Result<()> {
        self.screenshots_taken += 1;
        let file = format!(
            "{:02}_{}_{label}.png",
            self.step_index + 1,
            STEPS[self.step_index].name
        );
        self.game.screenshot(&self.output_directory.join(file))
    }

    /// Saves the game's state next to the screenshots, for working out why the current step
    /// failed. Does nothing if the game no longer answers.
    fn save_failure_snapshot(&mut self) -> Result<()> {
        let Ok(snapshot) = self.game.snapshot() else {
            return Ok(());
        };
        let file = self.output_directory.join(format!(
            "{:02}_{}_failure.json",
            self.step_index + 1,
            STEPS[self.step_index].name
        ));
        fs::write(&file, serde_json::to_string_pretty(&snapshot)?)
            .with_context(|| format!("writing {}", file.display()))
    }
}

/// Plays the fixed route from the start screen through death and restart, screenshotting each
/// step, and prints which steps passed. Stops at the first failed step.
pub(super) fn run(args: &PlaythroughArgs) -> Result<()> {
    enter_repository_root()?;
    if args.output_directory.exists() {
        fs::remove_dir_all(&args.output_directory)
            .with_context(|| format!("clearing {}", args.output_directory.display()))?;
    }
    fs::create_dir_all(&args.output_directory)
        .with_context(|| format!("creating {}", args.output_directory.display()))?;

    let log_file = args.output_directory.join("game.log");
    let game = ScriptedGame::launch(&log_file, args.remote)?;
    let mut playthrough = Playthrough {
        game,
        output_directory: args.output_directory.clone(),
        step_index: 0,
        screenshots_taken: 0,
    };

    let mut table = Table::new(["step", "result", "seconds", "notes"]);
    let mut failed = false;
    for (index, step) in STEPS.iter().enumerate() {
        playthrough.step_index = index;
        let screenshots_before = playthrough.screenshots_taken;
        let started = Instant::now();
        let outcome = (step.run)(&mut playthrough);
        if playthrough.screenshots_taken == screenshots_before {
            playthrough.screenshot("end")?;
        }
        let (result, notes) = match outcome {
            Ok(notes) => ("ok", notes),
            Err(error) => ("FAILED", format!("{error:#}")),
        };
        println!("{} {result}: {notes}", step.name);
        table.add_row(vec![
            step.name.to_owned(),
            result.to_owned(),
            format!("{:.1}", started.elapsed().as_secs_f64()),
            notes,
        ]);
        if result != "ok" {
            failed = true;
            playthrough.save_failure_snapshot()?;
            break;
        }
    }
    playthrough.game.exit()?;

    let problems = log_problems(&log_file)?;
    let (result, notes) = if problems.is_empty() {
        ("ok", "no errors or unexpected warnings".to_owned())
    } else {
        failed = true;
        (
            "FAILED",
            format!("{} problems, listed below", problems.len()),
        )
    };
    table.add_row(vec![
        "log".to_owned(),
        result.to_owned(),
        optional_cell(None),
        notes,
    ]);

    println!("\n{table}");
    for problem in &problems {
        println!("log: {problem}");
    }
    println!("screenshots in {}", args.output_directory.display());
    ensure!(!failed, "the playthrough failed");
    Ok(())
}

fn start(playthrough: &mut Playthrough) -> Result<String> {
    playthrough
        .game
        .wait_until("the start screen", STATE_TIMEOUT, |snapshot| {
            snapshot["state"] == "StartScreen"
        })?;
    thread::sleep(SCENE_SETTLE);
    playthrough.screenshot("title")?;

    let game = &mut playthrough.game;
    game.click_text("BEGIN", MouseButton::Primary)?;
    game.wait_until_playing(STATE_TIMEOUT)?;
    thread::sleep(SCENE_SETTLE);

    let snapshot = game.snapshot()?;
    Ok(format!(
        "{} interactables, {} portals",
        list(&snapshot["interactables"]).count(),
        list(&snapshot["portals"]).count()
    ))
}

fn equip_items(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    game.open_inventory()?;
    game.click_text(MAINHAND, MouseButton::Primary)?;
    game.click_text(OFFHAND, MouseButton::Primary)?;
    playthrough.screenshot("open")?;

    let game = &mut playthrough.game;
    let player = game.snapshot()?["player"].take();
    game.close_menu()?;

    ensure!(
        player["mainhand"] == MAINHAND && player["offhand"] == OFFHAND,
        "equipped {} and {}",
        player["mainhand"],
        player["offhand"]
    );
    Ok(format!("equipped {MAINHAND} and {OFFHAND}"))
}

fn pause(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    game.tap_key("Escape")?;
    game.wait_for_menu("Pause")?;

    let paused = game.snapshot()?;
    thread::sleep(PAUSE_WATCH);
    let still_paused = game.snapshot()?;
    playthrough.screenshot("menu")?;
    playthrough.game.close_menu()?;

    // The pause menu stops physics, not game time, so it checks that the villagers stand still.
    let distance_moved = |before: &Value| {
        let after = list(&still_paused["interactables"])
            .find(|after| after["entity"] == before["entity"])?;
        Some(distance(
            point(&before["position"])?,
            point(&after["position"])?,
        ))
    };
    let farthest_move = list(&paused["interactables"])
        .filter_map(distance_moved)
        .fold(0.0, f64::max);

    ensure!(
        farthest_move < 0.01,
        "a villager moved {farthest_move:.2} while paused"
    );
    Ok("villagers stood still while paused".to_owned())
}

/// Drinks the health potion and drops an item from the inventory screen, then picks the item
/// back up off the ground.
fn use_items(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    game.set_player_health(20.0, 100.0)?;
    game.open_inventory()?;
    game.click_text("Health Potion", MouseButton::Primary)?;
    game.click_text(DROPPED_ITEM, MouseButton::Secondary)?;
    let after_clicks = game.snapshot()?;
    game.close_menu()?;

    let hp = after_clicks["player"]["hp"].as_f64().unwrap_or_default();
    ensure!(hp > 20.0, "the potion left the player at {hp} hp");

    let has_item = |snapshot: &Value, name: &str| {
        list(&snapshot["player"]["inventory"]).any(|item| item["name"] == name)
    };
    ensure!(
        !has_item(&after_clicks, DROPPED_ITEM),
        "right-clicking {DROPPED_ITEM} kept it in the inventory"
    );

    let dropped_at = list(&after_clicks["items_on_ground"])
        .find(|item| item["name"] == DROPPED_ITEM)
        .and_then(|item| point(&item["position"]))
        .context("the dropped item is not on the ground")?;
    playthrough.screenshot("dropped")?;

    let game = &mut playthrough.game;
    game.walk_to(|_| Some(dropped_at), 6.0, Duration::from_secs(5))?;
    game.tap_key("Space")?;
    let picked_up = game.wait_until("the pickup", PICKUP_TIMEOUT, |snapshot| {
        has_item(snapshot, DROPPED_ITEM)
    })?;

    ensure!(
        !has_item(&picked_up, "Health Potion"),
        "the potion is still in the inventory"
    );
    Ok(format!(
        "potion took the player from 20 to {hp:.0} hp; dropped and picked up {DROPPED_ITEM}"
    ))
}

/// Walks up to every interactable and presses Space, noting which menu opened.
fn talk_to_npcs(playthrough: &mut Playthrough) -> Result<String> {
    let interactables = playthrough.game.snapshot()?["interactables"].take();
    ensure!(
        list(&interactables).next().is_some(),
        "no interactables in the hub"
    );

    let mut notes = Vec::new();
    let mut menus_opened = 0;

    for interactable in list(&interactables) {
        let name = interactable["name"].as_str().unwrap_or("unnamed");
        let position_now = |snapshot: &Value| {
            list(&snapshot["interactables"])
                .find(|found| found["entity"] == interactable["entity"])
                .and_then(|found| point(&found["position"]))
        };

        let game = &mut playthrough.game;
        let distance_left = game.walk_to(position_now, 12.0, Duration::from_secs(10))?;
        game.tap_key("Space")?;
        thread::sleep(INTERACT_WAIT);

        let menu = game.snapshot()?["menu"].take();
        let menu = menu.as_str().unwrap_or("?");
        if menu == "None" {
            notes.push(format!("{name}: nothing (stopped {distance_left:.0} away)"));
            continue;
        }

        menus_opened += 1;
        notes.push(format!("{name}: {menu}"));
        playthrough.screenshot(&name.to_lowercase())?;
        playthrough.game.close_menu()?;
    }

    ensure!(menus_opened > 0, "no interactable opened a menu: {notes:?}");
    Ok(notes.join(", "))
}

fn take_portal(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    let before = game.snapshot()?;
    let portal = point(&before["portals"][0]).context("no portal on this map")?;

    game.walk_to(|_| Some(portal), 4.0, Duration::from_secs(15))?;
    let after = game.wait_until("a new map", STATE_TIMEOUT, |snapshot| {
        is_playing(snapshot) && snapshot["map"]["entity"] != before["map"]["entity"]
    })?;
    thread::sleep(SCENE_SETTLE);

    Ok(format!(
        "{} enemies on the new map",
        list(&after["enemies"]).count()
    ))
}

/// Fights two warriors in the AI arena, attacking the nearest, with enough health to survive.
fn fight(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    game.load_map("AiTestArena", 1)?;
    game.set_player_health(1000.0, 1000.0)?;

    // Hits from before the fight don't count.
    let before = game.snapshot_with_new_hits()?;
    let player = &before["player"]["entity"];
    let start = point(&before["player"]["position"]).context("player has no position")?;
    let started_at = seconds(&before["time"]);

    let warriors = game.spawn_enemies(
        "Warrior",
        &[
            [start[0] + 120.0, start[1]],
            [start[0] - 120.0, start[1] + 40.0],
        ],
    )?;
    let is_warrior = |enemy: &&Value| {
        enemy["entity"]
            .as_u64()
            .is_some_and(|entity| warriors.contains(&entity))
    };

    let mut hits_landed = 0;
    let mut hits_taken = 0;
    let mut screenshot_taken = false;

    let warriors_left = loop {
        let snapshot = playthrough.game.snapshot_with_new_hits()?;
        let elapsed = seconds(&snapshot["time"]) - started_at;
        if elapsed >= FIGHT_SECONDS {
            break list(&snapshot["enemies"]).filter(is_warrior).count();
        }

        for hit in list(&snapshot["hits"]) {
            if hit["attacker"] == *player {
                hits_landed += 1;
            } else if hit["entity"] == *player {
                hits_taken += 1;
            }
        }

        let position = point(&snapshot["player"]["position"]).unwrap_or(start);
        let warrior_positions = list(&snapshot["enemies"])
            .filter(is_warrior)
            .filter_map(|warrior| point(&warrior["position"]));
        let input = match nearest(position, warrior_positions) {
            Some(warrior) => attack_input(position, warrior, FIGHT_REACH),
            None => json!({}),
        };
        playthrough.game.call("game/player_input", input)?;

        if !screenshot_taken && elapsed > FIGHT_SECONDS / 2.0 {
            screenshot_taken = true;
            playthrough.screenshot("mid")?;
        }
        thread::sleep(POLL_INTERVAL);
    };
    playthrough.game.release_player_input()?;

    ensure!(
        hits_landed > 0,
        "the player landed no hits in {FIGHT_SECONDS}s"
    );
    Ok(format!(
        "landed {hits_landed} hits, took {hits_taken}, {warriors_left}/2 warriors left"
    ))
}

fn die(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    let snapshot = game.snapshot()?;
    let [x, y] = point(&snapshot["player"]["position"]).context("player has no position")?;

    game.set_player_health(1.0, 100.0)?;
    game.spawn_enemies("Warrior", &[[x + 40.0, y], [x - 40.0, y], [x, y + 40.0]])?;
    game.wait_until("game over", STATE_TIMEOUT, |snapshot| {
        snapshot["state"] == "GameOver"
    })?;
    thread::sleep(SCENE_SETTLE);

    Ok("reached game over".to_owned())
}

fn restart(playthrough: &mut Playthrough) -> Result<String> {
    let game = &mut playthrough.game;
    game.click_text("Restart", MouseButton::Primary)?;
    let snapshot = game.wait_until_playing(STATE_TIMEOUT)?;
    thread::sleep(SCENE_SETTLE);

    Ok(format!(
        "back in {} with {} hp",
        snapshot["state"], snapshot["player"]["hp"]
    ))
}

fn seconds(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}
