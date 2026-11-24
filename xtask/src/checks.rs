use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs, thread};

use anyhow::{Context, Result, ensure};
use clap::Args;
use serde_json::{Value, json};

use crate::remote::{
    POLL_INTERVAL, Point, RemoteArgs, ScriptedGame, attack_input, distance, list, number, point,
    report_log_problems,
};
use crate::sweep::enter_repository_root;
use crate::table::Table;

/// Map recipe every check except the Swamp one fights in.
const TEST_ROOM: &str = "AiTestRoom";
/// Real time a new scene gets to finish spawning before a check looks at it.
const SCENE_SETTLE: Duration = Duration::from_millis(500);
/// Player health for checks the player must survive without being invincible.
const DURABLE_HEALTH: f64 = 100_000.0;
const WAIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Game seconds warriors fight before the pause, and real time the pause lasts.
const PAUSE_FIGHT_SECONDS: f64 = 2.0;
const PAUSE_WATCH: Duration = Duration::from_secs(3);
/// Mana the player raises the Knight Shield with, so it runs out in a few seconds.
const BLOCK_MANA: f64 = 10.0;
/// Mana below which a dropped shield counts as having run out. Mana regenerates between the drop
/// and the poll that sees it, so this allows a few tenths of a second's worth.
const OUT_OF_MANA: f64 = 3.0;
/// Game seconds a warrior swings at a still player.
const SWING_WATCH_SECONDS: f64 = 10.0;
/// Fraction of its health a warrior keeps so it drinks, yet survives one sword hit.
const DRINKING_HEALTH_FRACTION: f64 = 0.5;
/// Seconds an action source interrupted during its wind-up waits before it can be used again.
const WINDUP_LOCKOUT_SECONDS: f64 = 1.5;
/// Seconds the lockout may seem short by, since a drink's end is only seen at the next poll. The
/// potion's own cooldown is a second, so this still tells the lockout from it.
const POLL_TOLERANCE_SECONDS: f64 = 0.15;
/// Distance from the warrior the player steps to, past the distance at which it won't drink.
const DRINKING_DISTANCE: f64 = 300.0;
/// Game seconds enemies fight before the leak check finishes them off, and game seconds their
/// lights, statuses and hitboxes get to clean up after the last death.
const LEAK_FIGHT_SECONDS: f64 = 6.0;
const CLEANUP_SECONDS: f64 = 15.0;
/// Distance from an enemy at which the player swings at it.
const SWING_DISTANCE: f64 = 30.0;
const SWAMP_RECIPES: [&str; 2] = ["Swamp", "SwampWithALotOfEmptySquares"];
const SWAMP_SEED: u64 = 1;
/// Distance within which two enemies count as placed at the same spot.
const SAME_SPOT_DISTANCE: f64 = 16.0;
/// Real time the check keeps watching after game over, for a hit landing late.
const AFTER_GAME_OVER: Duration = Duration::from_secs(1);

/// Options for `checks`.
#[derive(Args)]
pub(super) struct ChecksArgs {
    #[command(flatten)]
    remote: RemoteArgs,

    #[arg(
        long,
        default_value = "runs/checks",
        help = "Where the game log goes; a rerun overwrites it"
    )]
    output_directory: PathBuf,
}

/// One check: it sets up a situation, then returns a note on what it saw or why it failed.
struct Check {
    name: &'static str,
    run: fn(&mut ScriptedGame) -> Result<String>,
}

/// The checks in the order they run. The corpse check ends in game over, so it runs last.
const CHECKS: &[Check] = &[
    Check {
        name: "pause",
        run: check_pause_freezes_fight,
    },
    Check {
        name: "held block",
        run: check_held_block_drains_mana,
    },
    Check {
        name: "committed swing",
        run: check_one_attack_per_swing,
    },
    Check {
        name: "wind-up lockout",
        run: check_windup_lockout,
    },
    Check {
        name: "leaks",
        run: check_fight_leaves_nothing,
    },
    Check {
        name: "swamp layouts",
        run: check_swamp_layouts_differ,
    },
    Check {
        name: "corpse",
        run: check_no_hits_after_death,
    },
];

/// Runs every check in one game and prints which passed, failing if any failed or the game
/// logged a problem.
pub(super) fn run(args: &ChecksArgs) -> Result<()> {
    enter_repository_root()?;
    fs::create_dir_all(&args.output_directory)
        .with_context(|| format!("creating {}", args.output_directory.display()))?;

    let log_file = args.output_directory.join("game.log");
    let mut game = ScriptedGame::launch(&log_file, args.remote)?;
    game.call("game/start", Value::Null)?;
    game.wait_until_playing(WAIT_TIMEOUT)?;
    thread::sleep(SCENE_SETTLE);
    game.equip_from_inventory(&["Sword"])?;

    let mut table = Table::new(["check", "result", "seconds", "notes"]);
    let mut failed_checks = 0;
    for check in CHECKS {
        let started = Instant::now();
        let (result, notes) = match (check.run)(&mut game) {
            Ok(notes) => ("ok", notes),
            Err(error) => {
                failed_checks += 1;
                ("FAILED", format!("{error:#}"))
            }
        };
        println!("{} {result}: {notes}", check.name);
        table.add_row(vec![
            check.name.to_owned(),
            result.to_owned(),
            format!("{:.1}", started.elapsed().as_secs_f64()),
            notes,
        ]);
    }
    game.exit()?;

    println!("\n{table}");
    report_log_problems(&[log_file])?;
    ensure!(failed_checks == 0, "{failed_checks} checks failed");
    Ok(())
}

/// Pauses warriors mid-fight: game time, every warrior and every hit must hold still, and time
/// must move again on resume.
fn check_pause_freezes_fight(game: &mut ScriptedGame) -> Result<String> {
    let [x, y] = enter_test_room(game)?;
    make_player_invincible(game)?;
    let warriors = game.spawn_enemies("Warrior", &[[x + 50.0, y], [x - 50.0, y], [x, y + 50.0]])?;
    wait_game_seconds(game, PAUSE_FIGHT_SECONDS)?;

    game.call("game/time", json!({ "paused": true }))?;
    game.wait_frames(2)?;
    let before = game.snapshot_with_new_hits()?;
    thread::sleep(PAUSE_WATCH);
    let during = game.snapshot_with_new_hits()?;
    game.call("game/time", json!({ "paused": false }))?;

    ensure!(
        during["time"] == before["time"],
        "game time moved from {} to {} while paused",
        before["time"],
        during["time"]
    );
    let hits_while_paused = list(&during["hits"]).count();
    ensure!(
        hits_while_paused == 0,
        "{hits_while_paused} hits landed while paused"
    );
    for &warrior in &warriors {
        let held = find_enemy(&before, warrior).context("a warrior is missing before the pause")?;
        let now = find_enemy(&during, warrior).context("a warrior is missing after the pause")?;
        let moved = match (point(&held["position"]), point(&now["position"])) {
            (Some(from), Some(to)) => distance(from, to),
            _ => 0.0,
        };
        ensure!(
            moved < 0.01,
            "warrior {warrior} moved {moved:.2} while paused"
        );
        ensure!(
            held["hp"] == now["hp"] && held["attacking"] == now["attacking"],
            "warrior {warrior} changed while paused: {held} then {now}"
        );
    }
    let paused_at = number(&before["time"]);
    game.wait_until("game time to resume", WAIT_TIMEOUT, |snapshot| {
        number(&snapshot["time"]) > paused_at
    })?;
    Ok(format!(
        "{} warriors held still through {:.0} s of pause",
        warriors.len(),
        PAUSE_WATCH.as_secs_f64()
    ))
}

/// Holds the Knight Shield up on little mana: it must drain mana while up and drop once mana
/// runs out, though the block stays held.
fn check_held_block_drains_mana(game: &mut ScriptedGame) -> Result<String> {
    enter_test_room(game)?;
    let given = game.call("game/give_item", json!({ "item": "knight_shield" }))?;
    let shield = given
        .as_u64()
        .context("game/give_item returned no entity")?;
    game.call(
        "game/use_item",
        json!({ "item": shield, "action": "equip" }),
    )?;
    game.call("game/set_player_mana", json!({ "mana": BLOCK_MANA }))?;
    game.call("game/player_input", json!({ "offhand": true }))?;

    let raised = game.wait_until("the shield to go up", WAIT_TIMEOUT, |snapshot| {
        is_item_in_use(snapshot, shield)
    })?;
    let dropped = game.wait_until("the shield to drop", WAIT_TIMEOUT, |snapshot| {
        !is_item_in_use(snapshot, shield)
    })?;
    game.release_player_input()?;
    game.call(
        "game/use_item",
        json!({ "item": shield, "action": "unequip" }),
    )?;

    let mana_raised = number(&raised["player"]["mana"]);
    let mana_dropped = number(&dropped["player"]["mana"]);
    ensure!(
        mana_dropped < OUT_OF_MANA,
        "the shield dropped with {mana_dropped:.1} mana left, not for lack of mana"
    );
    Ok(format!(
        "held {:.1} s on {mana_raised:.0} mana, dropped at {mana_dropped:.1}",
        number(&dropped["time"]) - number(&raised["time"])
    ))
}

/// Stands a warrior beside a still player: each run of its swing behavior, told apart by its
/// start time, must start at most one attack.
fn check_one_attack_per_swing(game: &mut ScriptedGame) -> Result<String> {
    let [x, y] = enter_test_room(game)?;
    game.set_player_health(DURABLE_HEALTH, DURABLE_HEALTH)?;
    let warrior = spawn_one(game, "Warrior", [x + 45.0, y])?;

    let started_at = number(&game.snapshot()?["time"]);
    let mut swing_started_at: Option<f64> = None;
    let mut swings = 0;
    let mut attacks_this_swing = 0;
    let mut most_attacks_in_a_swing = 0;
    let mut was_attacking = false;
    loop {
        let snapshot = game.snapshot()?;
        if number(&snapshot["time"]) - started_at >= SWING_WATCH_SECONDS {
            break;
        }
        let Some(warrior) = find_enemy(&snapshot, warrior) else {
            break;
        };
        let swing_start = warrior["behavior_start_times"]["swing"].as_f64();
        if swing_start.is_some() && swing_start != swing_started_at {
            swings += 1;
            attacks_this_swing = 0;
        }
        swing_started_at = swing_start;
        let attacking = warrior["attacking"] == true;
        if swing_start.is_some() && attacking && !was_attacking {
            attacks_this_swing += 1;
            most_attacks_in_a_swing = most_attacks_in_a_swing.max(attacks_this_swing);
        }
        was_attacking = attacking;
        thread::sleep(POLL_INTERVAL);
    }

    ensure!(
        swings > 0,
        "the warrior never swung in {SWING_WATCH_SECONDS} s"
    );
    ensure!(
        most_attacks_in_a_swing <= 1,
        "one swing started {most_attacks_in_a_swing} attacks"
    );
    Ok(format!("{swings} swings, at most one attack each"))
}

/// Hurts a warrior far from the player so it drinks its potion, knocks it back during the
/// drink's wind-up, then steps away: it must not start drinking again for the lockout.
fn check_windup_lockout(game: &mut ScriptedGame) -> Result<String> {
    let [x, y] = enter_test_room(game)?;
    make_player_invincible(game)?;
    // Facing away, the warrior doesn't see the player, so it drinks with no target.
    let spawned = game.call(
        "game/spawn_enemies",
        json!({ "kind": "Warrior", "positions": [[x + 400.0, y]], "facing": "right" }),
    )?;
    let warrior = list(&spawned)
        .find_map(Value::as_u64)
        .context("game/spawn_enemies returned no entity")?;
    let spawned_warrior = game.snapshot()?;
    let max_hp = number(&find_enemy(&spawned_warrior, warrior).context("no warrior")?["max_hp"]);
    let drinking_hp = DRINKING_HEALTH_FRACTION * max_hp;
    game.call(
        "game/set_enemy_health",
        json!({ "entity": warrior, "hp": drinking_hp }),
    )?;

    let drinking = game.wait_until("the warrior to drink", WAIT_TIMEOUT, |snapshot| {
        is_drinking(snapshot, warrior)
    })?;
    let position = find_enemy_position(&drinking, warrior)?;
    let beside = [position[0] - SWING_DISTANCE, position[1]];
    game.snapshot_with_new_hits()?;
    game.call("game/teleport_player", json!({ "position": beside }))?;
    game.call("game/player_input", attack_input(beside, position, 0.0))?;
    let stopped = game.wait_until("the drink to stop", WAIT_TIMEOUT, |snapshot| {
        !is_drinking(snapshot, warrior)
    })?;
    game.release_player_input()?;
    let knocked_back = list(&stopped["hits"]).any(|hit| hit["entity"].as_u64() == Some(warrior));
    ensure!(
        knocked_back,
        "the drink stopped before the player's hit landed"
    );
    let hp_after = number(&find_enemy(&stopped, warrior).context("the warrior died")?["hp"]);
    ensure!(
        hp_after <= drinking_hp,
        "the drink took effect ({drinking_hp:.0} to {hp_after:.0} hp) before the knockback"
    );

    // Far from the player again, the warrior drinks as soon as its potion allows.
    let knocked_to = find_enemy_position(&stopped, warrior)?;
    let away = [knocked_to[0] - DRINKING_DISTANCE, knocked_to[1]];
    game.call("game/teleport_player", json!({ "position": away }))?;
    let drinking_again =
        game.wait_until("the warrior to drink again", WAIT_TIMEOUT, |snapshot| {
            is_drinking(snapshot, warrior)
        })?;
    let wait = number(&drinking_again["time"]) - number(&stopped["time"]);
    ensure!(
        wait >= WINDUP_LOCKOUT_SECONDS - POLL_TOLERANCE_SECONDS,
        "the warrior drank again {wait:.2} s after the interrupted drink"
    );
    Ok(format!(
        "drank again {wait:.2} s after the interrupted drink"
    ))
}

/// Fights mages and a warrior, then kills them: the lights, status effects, hitboxes and action
/// uses must return to their counts before the fight.
fn check_fight_leaves_nothing(game: &mut ScriptedGame) -> Result<String> {
    let [x, y] = enter_test_room(game)?;
    make_player_invincible(game)?;
    let before = game.snapshot()?["entity_counts"].clone();
    let mut enemies = Vec::new();
    for (kind, position) in [
        ("FireMage", [x + 150.0, y]),
        ("IceMage", [x - 150.0, y]),
        ("Warrior", [x, y + 60.0]),
    ] {
        enemies.push(spawn_one(game, kind, position)?);
    }
    wait_game_seconds(game, LEAK_FIGHT_SECONDS)?;

    for &enemy in &enemies {
        game.call(
            "game/set_enemy_health",
            json!({ "entity": enemy, "hp": 1.0 }),
        )?;
        kill_enemy(game, enemy)?;
    }
    game.release_player_input()?;

    let cleanup_started_at = number(&game.snapshot()?["time"]);
    let after = game.wait_until("the fight to clean up", WAIT_TIMEOUT, |snapshot| {
        snapshot["entity_counts"] == before
            || number(&snapshot["time"]) - cleanup_started_at >= CLEANUP_SECONDS
    })?;
    let counts_after = &after["entity_counts"];
    ensure!(
        *counts_after == before,
        "{CLEANUP_SECONDS} s after the last death, counts are {counts_after} against {before} \
         before the fight"
    );
    Ok(format!(
        "{before} before and after {} deaths",
        enemies.len()
    ))
}

/// Builds both Swamp recipes from one seed: they must not place their enemies at the same spots.
fn check_swamp_layouts_differ(game: &mut ScriptedGame) -> Result<String> {
    game.release_player_input()?;
    let mut layouts = Vec::new();
    for recipe in SWAMP_RECIPES {
        game.load_map(recipe, SWAMP_SEED)?;
        game.wait_frames(2)?;
        let snapshot = game.snapshot()?;
        let positions: Vec<Point> = list(&snapshot["enemies"])
            .filter_map(|enemy| point(&enemy["position"]))
            .collect();
        layouts.push(positions);
    }
    let [first, second] = [&layouts[0], &layouts[1]];
    let shared_spots = first
        .iter()
        .filter(|&&spot| {
            second
                .iter()
                .any(|&other| distance(spot, other) < SAME_SPOT_DISTANCE)
        })
        .count();
    ensure!(
        !(first.len() == second.len() && shared_spots == first.len()),
        "both recipes placed their {} enemies at the same spots for seed {SWAMP_SEED}",
        first.len()
    );
    Ok(format!(
        "{} and {} enemies, {shared_spots} at shared spots",
        first.len(),
        second.len()
    ))
}

/// Lets warriors kill a 1 hp player: no hit may land on the player after the frame of the
/// killing hit.
fn check_no_hits_after_death(game: &mut ScriptedGame) -> Result<String> {
    let [x, y] = enter_test_room(game)?;
    let player = game.snapshot_with_new_hits()?["player"]["entity"].clone();
    game.set_player_health(1.0, 100.0)?;
    game.spawn_enemies("Warrior", &[[x + 40.0, y], [x - 40.0, y], [x, y + 40.0]])?;

    let mut hit_times = Vec::new();
    let mut game_over_since: Option<Instant> = None;
    let deadline = Instant::now() + WAIT_TIMEOUT;
    while game_over_since.is_none_or(|since| since.elapsed() < AFTER_GAME_OVER) {
        ensure!(Instant::now() < deadline, "the player didn't die");
        let snapshot = game.snapshot_with_new_hits()?;
        for hit in list(&snapshot["hits"]).filter(|hit| hit["entity"] == player) {
            hit_times.push(number(&hit["time"]));
        }
        if snapshot["state"] == "GameOver" && game_over_since.is_none() {
            game_over_since = Some(Instant::now());
        }
        thread::sleep(POLL_INTERVAL);
    }

    let killed_at = hit_times
        .iter()
        .copied()
        .min_by(f64::total_cmp)
        .context("the player reached game over without being hit")?;
    let hits_on_corpse = hit_times.iter().filter(|&&time| time > killed_at).count();
    ensure!(
        hits_on_corpse == 0,
        "{hits_on_corpse} hits landed on the player after the killing one"
    );
    Ok("no hit landed after the frame of the killing hit".to_owned())
}

/// Loads the test room fresh, with time running, the player at full health and mana, vulnerable
/// and holding no input, and returns the player's position.
fn enter_test_room(game: &mut ScriptedGame) -> Result<Point> {
    game.release_player_input()?;
    game.call("game/time", json!({ "paused": false }))?;
    game.load_map(TEST_ROOM, 1)?;
    thread::sleep(SCENE_SETTLE);
    game.set_player_health(100.0, 100.0)?;
    game.call("game/set_player_mana", json!({ "mana": 100.0 }))?;
    game.call("game/set_invincible", json!({ "invincible": false }))?;
    point(&game.snapshot()?["player"]["position"]).context("player has no position")
}

fn make_player_invincible(game: &mut ScriptedGame) -> Result<()> {
    game.call("game/set_invincible", json!({ "invincible": true }))?;
    Ok(())
}

fn wait_game_seconds(game: &mut ScriptedGame, seconds: f64) -> Result<()> {
    let started_at = number(&game.snapshot()?["time"]);
    game.wait_until(
        &format!("{seconds} game seconds"),
        WAIT_TIMEOUT,
        |snapshot| number(&snapshot["time"]) - started_at >= seconds,
    )?;
    Ok(())
}

fn spawn_one(game: &mut ScriptedGame, kind: &str, position: Point) -> Result<u64> {
    game.spawn_enemies(kind, &[position])?
        .first()
        .copied()
        .with_context(|| format!("game/spawn_enemies returned no {kind}"))
}

/// Teleports the player beside `enemy` and swings at it until it dies.
fn kill_enemy(game: &mut ScriptedGame, enemy: u64) -> Result<()> {
    let deadline = Instant::now() + WAIT_TIMEOUT;
    loop {
        let snapshot = game.snapshot()?;
        let Some(found) = find_enemy(&snapshot, enemy) else {
            return Ok(());
        };
        if found["dead"] == true {
            return Ok(());
        }
        ensure!(
            Instant::now() < deadline,
            "enemy {enemy} survived the swings"
        );
        let position = find_enemy_position(&snapshot, enemy)?;
        let beside = [position[0] - SWING_DISTANCE, position[1]];
        game.call("game/teleport_player", json!({ "position": beside }))?;
        game.call("game/player_input", attack_input(beside, position, 0.0))?;
        thread::sleep(Duration::from_millis(200));
    }
}

fn find_enemy(snapshot: &Value, entity: u64) -> Option<&Value> {
    list(&snapshot["enemies"]).find(|enemy| enemy["entity"].as_u64() == Some(entity))
}

fn find_enemy_position(snapshot: &Value, entity: u64) -> Result<Point> {
    find_enemy(snapshot, entity)
        .and_then(|enemy| point(&enemy["position"]))
        .with_context(|| format!("enemy {entity} has no position"))
}

/// Whether one of the warrior's running behaviors is a heal.
fn is_drinking(snapshot: &Value, warrior: u64) -> bool {
    find_enemy(snapshot, warrior).is_some_and(|warrior| {
        list(&warrior["behaviors"])
            .filter_map(Value::as_str)
            .any(|name| name.starts_with("heal"))
    })
}

fn is_item_in_use(snapshot: &Value, item: u64) -> bool {
    list(&snapshot["player"]["inventory"])
        .any(|found| found["entity"].as_u64() == Some(item) && found["in_use"] == true)
}
