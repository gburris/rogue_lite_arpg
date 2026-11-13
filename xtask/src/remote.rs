use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{env, thread};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};

use crate::process::{ChildProcess, run_to_completion};

/// Where the game's `RemoteHttpPlugin` listens.
const ADDRESS: &str = "127.0.0.1:15702";
/// Time a started game gets to open its remote server.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(300);
/// Time a closed game gets to exit before it is killed.
const EXIT_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAP_LOAD_TIMEOUT: Duration = Duration::from_secs(120);
const MENU_TIMEOUT: Duration = Duration::from_secs(5);
const SCREENSHOT_TIMEOUT: Duration = Duration::from_secs(10);
/// Pause between snapshots while waiting on the game, and between steering updates.
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(50);
/// Frames the UI gets to lay out a menu or react to a click before the next key or click.
const UI_SETTLE_FRAMES: u64 = 10;

/// A 2D point or vector as the game's JSON writes it.
pub(super) type Point = [f64; 2];

/// Which mouse button a click presses.
#[derive(Clone, Copy, Debug)]
pub(super) enum MouseButton {
    Primary,
    Secondary,
}

/// A dev build of the game started for a script and driven over the Bevy Remote Protocol with
/// the `game/…` methods in `src/dev/remote/`. Killed if still running when dropped.
pub(super) struct ScriptedGame {
    process: ChildProcess,
    next_request_id: u64,
    last_hit_read: u64,
}

impl ScriptedGame {
    /// Builds and starts the dev game, logging to `log_file`, and waits for its remote server.
    pub(super) fn launch(log_file: &Path) -> Result<Self> {
        ensure!(
            send_request(ADDRESS, &request_body(0, "game/snapshot", &Value::Null)).is_err(),
            "a game is already serving the remote protocol on {ADDRESS}; close it first"
        );

        let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        run_to_completion("cargo build", Command::new(&cargo).arg("build"))?;

        let log =
            File::create(log_file).with_context(|| format!("creating {}", log_file.display()))?;
        let process = ChildProcess::spawn(
            "the game",
            Command::new(&cargo)
                .args(["run", "--quiet"])
                .stdout(log.try_clone()?)
                .stderr(log),
        )?;
        let mut game = Self {
            process,
            next_request_id: 0,
            last_hit_read: 0,
        };

        let deadline = Instant::now() + STARTUP_TIMEOUT;
        while game.snapshot().is_err() {
            ensure!(
                !game.process.has_exited(),
                "the game exited during startup; see {}",
                log_file.display()
            );
            ensure!(
                Instant::now() < deadline,
                "the game's remote server did not answer"
            );
            thread::sleep(Duration::from_millis(500));
        }
        Ok(game)
    }

    /// Calls a remote method; `Value::Null` sends no params.
    pub(super) fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next_request_id += 1;
        let body = request_body(self.next_request_id, method, &params);

        let response = send_request(ADDRESS, &body).with_context(|| format!("calling {method}"))?;
        let mut response: Value = serde_json::from_str(&response)
            .with_context(|| format!("{method} answered with invalid JSON: {response}"))?;

        if let Some(error) = response.get("error") {
            bail!("{method} failed: {error}");
        }
        Ok(response["result"].take())
    }

    /// The game's state now. Its `hits` are every hit since the last
    /// [`Self::snapshot_with_new_hits`], so looking at the game never loses one.
    pub(super) fn snapshot(&mut self) -> Result<Value> {
        self.call("game/snapshot", json!({ "hits_after": self.last_hit_read }))
    }

    /// A snapshot whose `hits` are the ones landed since the last call of this method.
    pub(super) fn snapshot_with_new_hits(&mut self) -> Result<Value> {
        let snapshot = self.snapshot()?;
        self.last_hit_read = snapshot["last_hit"]
            .as_u64()
            .context("the snapshot has no `last_hit`")?;
        Ok(snapshot)
    }

    /// Polls snapshots until `is_done` accepts one, failing after `timeout` of real time.
    pub(super) fn wait_until(
        &mut self,
        what: &str,
        timeout: Duration,
        is_done: impl Fn(&Value) -> bool,
    ) -> Result<Value> {
        let deadline = Instant::now() + timeout;
        loop {
            let snapshot = self.snapshot()?;
            if is_done(&snapshot) {
                return Ok(snapshot);
            }
            ensure!(
                Instant::now() < deadline,
                "timed out after {timeout:?} waiting for {what}; last state {}, menu {}",
                snapshot["state"],
                snapshot["menu"]
            );
            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Waits until the game has drawn `frames` more frames.
    pub(super) fn wait_frames(&mut self, frames: u64) -> Result<()> {
        let frame_of = |snapshot: &Value| snapshot["frame"].as_u64().unwrap_or_default();

        let last_frame = frame_of(&self.snapshot()?) + frames;
        self.wait_until(&format!("{frames} frames"), REQUEST_TIMEOUT, |snapshot| {
            frame_of(snapshot) >= last_frame
        })?;
        Ok(())
    }

    /// Waits for the player to exist in the `Playing` state.
    pub(super) fn wait_until_playing(&mut self, timeout: Duration) -> Result<Value> {
        self.wait_until("Playing with a player", timeout, is_playing)
    }

    /// Waits until `menu` is the open menu, then for its UI to be laid out. `"None"` waits for
    /// every menu to close.
    pub(super) fn wait_for_menu(&mut self, menu: &str) -> Result<()> {
        self.wait_until(&format!("menu {menu}"), MENU_TIMEOUT, |snapshot| {
            snapshot["menu"] == menu
        })?;
        self.wait_frames(UI_SETTLE_FRAMES)
    }

    /// Builds a map recipe and waits until the new map is playing with a player on it.
    pub(super) fn load_map(&mut self, recipe: &str, seed: u64) -> Result<Value> {
        let previous_map = self.snapshot()?["map"]["entity"].take();

        self.call("game/load_map", json!({ "recipe": recipe, "seed": seed }))?;
        self.wait_until(&format!("{recipe} to load"), MAP_LOAD_TIMEOUT, |snapshot| {
            snapshot["map"]["entity"] != previous_map && is_playing(snapshot)
        })
    }

    pub(super) fn set_player_health(&mut self, hp: f64, max_hp: f64) -> Result<()> {
        self.call(
            "game/set_player_health",
            json!({ "hp": hp, "max_hp": max_hp }),
        )?;
        Ok(())
    }

    /// Spawns one enemy of `kind` at each position and returns their entities.
    pub(super) fn spawn_enemies(&mut self, kind: &str, positions: &[Point]) -> Result<Vec<u64>> {
        let spawned = self.call(
            "game/spawn_enemies",
            json!({ "kind": kind, "positions": positions }),
        )?;
        Ok(list(&spawned).filter_map(Value::as_u64).collect())
    }

    /// Lets go of everything `game/player_input` holds on the player.
    pub(super) fn release_player_input(&mut self) -> Result<()> {
        self.call("game/player_input", json!({}))?;
        Ok(())
    }

    pub(super) fn tap_key(&mut self, key: &str) -> Result<()> {
        self.call("game/key", json!({ "key": key, "action": "tap" }))?;
        Ok(())
    }

    /// Clicks the center of the visible UI text that reads `text`, ignoring case, then gives
    /// the UI time to react.
    pub(super) fn click_text(&mut self, text: &str, button: MouseButton) -> Result<()> {
        let texts = self.call("game/ui", Value::Null)?;
        let reads_as_text = |found: &&Value| {
            found["text"]
                .as_str()
                .is_some_and(|found| found.trim().eq_ignore_ascii_case(text))
        };
        let Some(found) = list(&texts).find(reads_as_text) else {
            let visible: Vec<&str> = list(&texts)
                .filter_map(|visible| visible["text"].as_str())
                .collect();
            bail!("no UI text {text:?}; visible: {visible:?}");
        };

        let button = match button {
            MouseButton::Primary => "primary",
            MouseButton::Secondary => "secondary",
        };
        self.call(
            "game/click",
            json!({ "position": found["center"], "button": button }),
        )?;
        self.wait_frames(UI_SETTLE_FRAMES)
    }

    pub(super) fn open_inventory(&mut self) -> Result<()> {
        self.tap_key("KeyI")?;
        self.wait_for_menu("Inventory")
    }

    /// Opens the inventory, clicks each item to equip it, and closes the inventory again.
    pub(super) fn equip_from_inventory(&mut self, items: &[&str]) -> Result<()> {
        self.open_inventory()?;
        for item in items {
            self.click_text(item, MouseButton::Primary)?;
        }
        self.close_menu()
    }

    /// Presses Escape and waits for the menu to close.
    pub(super) fn close_menu(&mut self) -> Result<()> {
        self.tap_key("Escape")?;
        self.wait_for_menu("None")
    }

    /// Steers the player straight at the point `target` finds in each snapshot, so it can follow
    /// something that moves, until within `tolerance`, then lets go. Returns the distance left,
    /// which is above `tolerance` when `timeout` ran out or the target vanished first.
    pub(super) fn walk_to(
        &mut self,
        target: impl Fn(&Value) -> Option<Point>,
        tolerance: f64,
        timeout: Duration,
    ) -> Result<f64> {
        let deadline = Instant::now() + timeout;
        let mut distance_left = f64::INFINITY;

        while Instant::now() < deadline {
            let snapshot = self.snapshot()?;
            let (Some(position), Some(target)) =
                (point(&snapshot["player"]["position"]), target(&snapshot))
            else {
                break;
            };

            distance_left = distance(position, target);
            if distance_left < tolerance {
                break;
            }

            let direction = offset(position, target);
            self.call("game/player_input", json!({ "move": direction }))?;
            thread::sleep(POLL_INTERVAL);
        }

        self.release_player_input()?;
        Ok(distance_left)
    }

    /// Saves the window to `path` and waits for the file to appear.
    pub(super) fn screenshot(&mut self, path: &Path) -> Result<()> {
        let path = env::current_dir()?.join(path);
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        }

        self.call(
            "game/screenshot",
            json!({ "path": path.to_string_lossy().replace('\\', "/") }),
        )?;

        let deadline = Instant::now() + SCREENSHOT_TIMEOUT;
        while !path.exists() {
            ensure!(
                Instant::now() < deadline,
                "the game did not save {}",
                path.display()
            );
            self.wait_frames(1)?;
        }
        Ok(())
    }

    /// Asks the game to quit and waits for it, killing it if it hangs.
    pub(super) fn exit(mut self) -> Result<()> {
        // The game may close the connection before answering.
        let _ = self.call("game/exit", Value::Null);
        self.process.wait_or_kill(EXIT_TIMEOUT)
    }
}

/// Whether a snapshot shows the player in the `Playing` state.
pub(super) fn is_playing(snapshot: &Value) -> bool {
    snapshot["state"] == "Playing" && !snapshot["player"].is_null()
}

/// The elements of a JSON list, or nothing when the value isn't one.
pub(super) fn list(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
}

/// The position nearest to `from`.
pub(super) fn nearest(from: Point, positions: impl Iterator<Item = Point>) -> Option<Point> {
    positions.min_by(|a, b| distance(from, *a).total_cmp(&distance(from, *b)))
}

/// Player input that aims and swings at `enemy`, closing in while it is farther than `reach`.
pub(super) fn attack_input(position: Point, enemy: Point, reach: f64) -> Value {
    let aim = offset(position, enemy);

    if length(aim) > reach {
        json!({ "aim": aim, "move": aim, "mainhand": true })
    } else {
        json!({ "aim": aim, "mainhand": true })
    }
}

/// A JSON `[x, y]` as a point.
pub(super) fn point(value: &Value) -> Option<Point> {
    Some([value.get(0)?.as_f64()?, value.get(1)?.as_f64()?])
}

pub(super) fn length(vector: Point) -> f64 {
    vector[0].hypot(vector[1])
}

/// The vector from one point to another.
pub(super) fn offset(from: Point, to: Point) -> Point {
    [to[0] - from[0], to[1] - from[1]]
}

pub(super) fn distance(from: Point, to: Point) -> f64 {
    length(offset(from, to))
}

fn request_body(id: u64, method: &str, params: &Value) -> String {
    let mut body = json!({ "jsonrpc": "2.0", "id": id, "method": method });
    if !params.is_null() {
        body["params"] = params.clone();
    }
    body.to_string()
}

/// Posts `body` over plain HTTP/1.1 and returns the response body.
fn send_request(address: &str, body: &str) -> Result<String> {
    let mut stream = TcpStream::connect(address)?;
    stream.set_read_timeout(Some(REQUEST_TIMEOUT))?;
    write!(
        stream,
        "POST / HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let (head, content) = response
        .split_once("\r\n\r\n")
        .context("HTTP response without a header end")?;
    ensure!(
        head.starts_with("HTTP/1.1 200"),
        "HTTP error: {}",
        head.lines().next().unwrap_or_default()
    );
    if head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        decode_chunked(content)
    } else {
        Ok(content.to_owned())
    }
}

fn decode_chunked(mut content: &str) -> Result<String> {
    let mut decoded = String::new();
    loop {
        let (size, rest) = content
            .split_once("\r\n")
            .context("chunk without a size line")?;
        let size = usize::from_str_radix(size.trim(), 16).context("bad chunk size")?;
        if size == 0 {
            return Ok(decoded);
        }
        ensure!(rest.len() >= size, "chunk shorter than its size");
        decoded.push_str(&rest[..size]);
        content = rest[size..].trim_start_matches("\r\n");
    }
}
