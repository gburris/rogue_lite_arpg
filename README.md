![image](https://github.com/user-attachments/assets/8e19637c-eb1f-4170-b202-663c1434e073)

# Build and run

Run all commands from the repository root. They work in bash and PowerShell. Run Cargo commands
one at a time, because each one locks the build directory.

`rust-toolchain.toml` pins a nightly toolchain. `rustup` installs it on the first build.

## Builds

| Build | Command | Use it for |
| --- | --- | --- |
| Dev | `cargo run` or `bevy run` | Daily work: hot reload, dev keys, F3 overlay |
| Release | `cargo run --release` | Playing at full speed |
| Profiling | `cargo tracy` | Performance measurement: release speed, debug symbols, Tracy spans |
| Web dev | `bevy run web` | Testing in the browser |
| Web release | `bevy build --release web --bundle` | Publishing to the web |

`cargo tracy` is an alias in `.cargo/config.toml` for
`cargo run --profile profiling --no-default-features --features tracy`.

Web builds need the [Bevy CLI](https://github.com/TheBevyFlock/bevy_cli) and the wasm target:
`rustup target add wasm32-unknown-unknown`.

## Map seed

The `MAP_SEED` environment variable fixes the map layout and enemy placement:

```sh
MAP_SEED=7 cargo run                                       # bash
$env:MAP_SEED = "7"; cargo run; Remove-Item Env:MAP_SEED   # PowerShell
```

## Local Cargo settings

Put machine-specific Cargo settings, such as a faster linker, in `.cargo/config.local.toml`. Git
ignores this file, and `.cargo/config.toml` loads it if it exists.

## Check

| Command | Checks |
| --- | --- |
| `cargo check --all-targets` | Dev features |
| `cargo check --all-targets --no-default-features` | Release features |
| `cargo check --all-targets --no-default-features --features tracy` | Profiling features |
| `cargo clippy --workspace --all-targets` | Lints for the game and `xtask` |

# Performance tools

Measure with the profiling build. Dev build timings are not accurate.

| Question | Tool |
| --- | --- |
| How fast is a large fight? | [`cargo xtask sweep`](#sweep) |
| Did a change make the game faster or slower? | [`cargo xtask compare`](#compare) on two sweeps |
| Which systems use the time? | [`cargo xtask perf-report`](#perf-report) |
| What happens in one slow frame? | [Tracy](#tracy) |
| What are the live counts while I play? | F3 overlay (dev build) |
| In what order do the systems run? | [Schedule graphs](#schedule-graphs) |

All output goes to `runs/`. Git ignores this folder.

## Compare a change

1. On the old code, record a baseline:

   ```sh
   cargo xtask sweep --counts 500,1000 --repeats 3 --milestone before
   ```

2. Make the change. Record again:

   ```sh
   cargo xtask sweep --counts 500,1000 --repeats 3 --milestone after
   ```

3. Compare frame times:

   ```sh
   cargo xtask compare runs/milestones/<date>_before runs/milestones/<date>_after
   ```

4. Find the systems that changed:

   ```sh
   cargo xtask perf-report runs/milestones/<date>_before runs/milestones/<date>_after
   ```

For a fair comparison:

- Use the same machine, `--map-seed` and `--engagement` for both sweeps.
- Close other programs that use the CPU or GPU.
- Make sure the Engaged and Projectiles columns in `compare` are similar. If they are not, the
  sweeps measured different fights.
- Check the `git` and `dirty` fields in each summary line. They show which code each sweep ran.

## Stress arena

```
cargo run   -- --stress <enemies> [options]   # dev build
cargo tracy -- --stress <enemies> [options]   # profiling build
```

Starts the game in a small arena. The player is invincible, and the enemies surround the player.
The game divides the enemies equally between the enemy kinds and drops the remainder. The start
screen is skipped, and vsync is off.

| Option | Default | Description |
| --- | --- | --- |
| `--stress <n>` | Normal game | Total enemies |
| `--seconds <n>` | Run until closed | After a 5 s warm-up, measure for this many seconds, write a summary line, then quit |
| `--engagement <mode>` | `all` | How enemies find the player. See below |
| `--player <script>` | `still` | How the player moves: `still`, `circle` or `strafe` |

| `--engagement` | Enemies | Use it for |
| --- | --- | --- |
| `all` | Target the player from spawn | Repeatable A/B runs. Every run has the same fight |
| `sight` | Attack when they see the player, as in normal play | Realistic fights. The load changes between runs |
| `none` | Cannot see and only wander | The cost of the crowd without combat |

With `--seconds`, the game writes one summary line to the log and appends it to
`runs/stress_summary.txt`. The line contains:

| Fields | Meaning |
| --- | --- |
| `arena`, `engagement`, `player`, `seconds` | The run settings |
| `startup_seconds` | Time from the first frame to the arena: asset loading and map spawn |
| `git`, `dirty` | Commit and uncommitted changes when the run ends |
| `optimized`, `tracy` | `false` for a dev build; `true` when Tracy spans are compiled in |
| `cpu` | CPU name |
| `frames`, `median_frame_ms`, `p99_frame_ms`, `max_frame_ms` | Frame times |
| `mean_game/...` | The mean of each `game/...` diagnostic, such as `mean_game/ai/engaged` |

A `-` means that the value was not available.

```sh
cargo run -- --stress 1000                                  # dev build, run until closed
cargo run -- --stress 1000 --seconds 60 --engagement none   # timed, crowd only
cargo run -- --stress 500 --player circle                   # moving player and camera
cargo tracy -- --stress 1000                                # profiling build
```

## Tracy

```sh
mkdir runs                                  # once
tracy-capture -o runs/baseline.tracy -f     # terminal 1: start the capture first
cargo tracy                                 # terminal 2: play, then close the game
tracy-profiler runs/baseline.tracy          # open the capture
```

Run `tracy-profiler` with no file and click Connect to see a live view.

**Requires:** Tracy v0.14.0 tools on `PATH`. This version matches `tracy-client` 0.18.5.

A capture contains:

- A plot for each `game/...` diagnostic.
- A message for each state change, such as `AppState: StartScreen => AssetLoading`.
- A message for each log line, such as `hitch: ...`.

The profiling build has no console. To read a panic, redirect stderr:
`target/profiling/baba_yaga 2> panic.log`.

## xtask

```
cargo xtask <sweep|compare|perf-report|playthrough|ai-metrics|map-sweep> [options]
```

Development commands written in Rust. Run them from any folder in the repository. Add `--help`
to see all options. The last three play the game through its remote protocol; see
[Scripted play tools](#scripted-play-tools).

### sweep

```
cargo xtask sweep [options]
```

Builds the profiling build once. Then records one Tracy capture for each engagement, enemy count
and repeat in the stress arena. Each run quits automatically.

**Requires:** `tracy-capture` on `PATH`.

| Option | Default | Description |
| --- | --- | --- |
| `--counts <n,n,...>` | `100,200,500,1000` | Total enemies for each run |
| `--repeats <n>` | `1` | Runs for each count and engagement. Use 2 or 3; `compare` shows the median |
| `--engagement <mode,...>` | `all` | One or more of `all`, `sight` and `none` |
| `--seconds <n>` | `30` | Measured seconds for each run, after the 5 s warm-up |
| `--capture-seconds <n>` | Whole run | Stops each capture early. Use 15 to 20 if you export events: a 30 s capture exports approximately 2.5 GB |
| `--map-seed <n>` | `1` | Map layout and enemy placement |
| `--milestone <name>` | None | Writes to `runs/milestones/<date>_<name>`. Later sweeps do not overwrite it |
| `--output-directory <path>` | `runs/stress` | Output folder without `--milestone`. The next sweep overwrites it |

**Output:** in the output folder, one `<engagement>_<count>_r<repeat>.tracy` capture and one
`.log` for each run, and a `summary.txt` with the summary lines. At the end, the sweep prints the
`compare` table.

```sh
cargo xtask sweep
cargo xtask sweep --counts 500,1000 --repeats 3 --capture-seconds 20
cargo xtask sweep --counts 1000 --engagement all,none --milestone substeps
```

### compare

```
cargo xtask compare <path>... [--outlier-fraction <fraction>]
```

Shows a table of summary lines. Each row is one source, enemy count, engagement and player
script.

| Option | Default | Description |
| --- | --- | --- |
| `<path>...` | Required | Summary files, or folders that contain a `summary.txt` (a sweep or a milestone) |
| `--outlier-fraction <f>` | `0.25` | How far a run's projectile count can be from its row's median before the run is an outlier |

**Output:** for each row, the median of the median and p99 frame times, the slowest frame, and
the engaged brains, projectiles and lights. Then a list of outlier runs. An outlier measured a
different fight, so do not use it in an A/B comparison.

```sh
cargo xtask compare runs/stress
cargo xtask compare runs/milestones/2026-11-01_before runs/milestones/2026-11-03_after
```

### perf-report

```
cargo xtask perf-report <path>...
```

Exports Tracy captures with `tracy-csvexport`. Shows the time per frame for the important zones:
main and render threads, fixed step, physics, sight raycasts, lights, transform propagation,
decisions and appliers. The fixed step also shows time per call, because it does not run every
frame.

**Requires:** `tracy-csvexport` on `PATH`.

| Option | Default | Description |
| --- | --- | --- |
| `<path>...` | Required | `.tracy` files, or folders that contain them |

**Output:** one row for each zone and one column for each capture.

- The frame count includes warm-up and loading. Only compare captures that you made the same
  way.
- Each capture adds a column. Use few captures, for example one count with 2 or 3 repeats.

```sh
cargo xtask perf-report runs/stress
cargo xtask perf-report runs/baseline.tracy runs/stress/all_1000_r1.tracy
```

## Schedule graphs

```sh
cargo run -- --dump-schedules
dot -Tsvg runs/schedules/update.dot -o runs/schedules/update.svg
```

The dev build writes `runs/schedules/update.dot` and `runs/schedules/fixed_update.dot`, then
quits.

**Requires:** Graphviz (`dot`) on `PATH` to make the images.

# Scripted play tools

These tools start the dev build and play it through the Bevy Remote Protocol, so a script or an
AI agent can test the game without a person at the keyboard.

| Question | Tool |
| --- | --- |
| Does the game still work from the start screen to a restart? | [`cargo xtask playthrough`](#playthrough) |
| Did a change make the enemies fight differently? | [`cargo xtask ai-metrics`](#ai-metrics) on the old and new code |
| Do the map recipes build without stuck enemies? | [`cargo xtask map-sweep`](#map-sweep) |
| Why did an enemy choose that behavior? | `game/explain` in the [remote protocol](#remote-protocol) |
| How do I try one thing by hand, or from my own script? | The [remote protocol](#remote-protocol) |

All output goes to `runs/`.

## Check a change

1. After any gameplay or UI change, play the fixed route:

   ```sh
   cargo xtask playthrough
   ```

   Every step must show `ok`, including the `log` row.

2. For an AI change, record the old code first, then the new code:

   ```sh
   cargo xtask ai-metrics    # on the old code
   cargo xtask ai-metrics    # on the new code; prints changes against the first run
   ```

3. For a map recipe or placement change, build many seeds:

   ```sh
   cargo xtask map-sweep --seeds 20
   ```

For a fair `ai-metrics` comparison:

- Use the same `--seconds`, `--enemies`, `--repeats` and `--speed` for both runs.
- Trust only large changes. Enemy decisions are random, so two runs of the same code differ.
- Check which file the changes are against. The command prints its name.

## Remote protocol

The native dev build serves the Bevy Remote Protocol on `127.0.0.1:15702`. It adds `game/...`
methods to Bevy's built-in `world....` methods.

```sh
cargo run -- --remote-port 15702
curl -d '{"jsonrpc":"2.0","id":1,"method":"game/help"}' 127.0.0.1:15702
```

| Method | Use it to |
| --- | --- |
| `game/help` | List every `game/...` method with its params |
| `game/snapshot` | Read the state: player, enemies, items, menus, map and recent hits |
| `game/player_input`, `game/key`, `game/click`, `game/ui` | Play: move, aim, attack, press keys and click UI text |
| `game/load_map`, `game/spawn_enemies`, `game/set_player_health`, `game/give_item` and others | Set up a test |
| `game/explain` | See how a brain scored each behavior at its last decision |
| `game/time`, `game/screenshot`, `game/exit` | Control the session |

Use a different `--remote-port` for each game to run several at the same time.

## playthrough

```
cargo xtask playthrough [options]
```

Plays a fixed route: start screen, equip a sword and shield, pause, use and drop items, talk to
every villager, take the portal, fight two warriors, die and restart. The run stops at the first
failed step.

| Option | Default | Description |
| --- | --- | --- |
| `--output-directory <path>` | `runs/playthrough` | Output folder. The next run overwrites it |
| `--remote-port <n>` | `15702` | Port of the game's remote protocol server |

**Output:** a table with one row for each step, and a `log` row that fails if the game logged an
error, a panic or an unexpected warning. In the output folder, a screenshot for each step, the
`game.log`, and a `<step>_failure.json` snapshot of the game state if a step failed.

```sh
cargo xtask playthrough
cargo xtask playthrough --remote-port 15710
```

## ai-metrics

```
cargo xtask ai-metrics [options]
```

Fights each enemy kind against each player script in an empty room. Measures how the enemies
moved and fought, and compares the result with an earlier run.

| Option | Default | Description |
| --- | --- | --- |
| `--kinds <kind,...>` | `Warrior,IceMage,FireMage,Bat` | Enemy kinds |
| `--scripts <script,...>` | `still,strafe,fight` | What the player does: stands, strafes, or attacks the nearest enemy |
| `--enemies <n>` | `3` | Enemies for each scenario |
| `--seconds <n>` | `20` | Game seconds for each scenario |
| `--repeats <n>` | `3` | Runs for each scenario. The table shows the median |
| `--speed <n>` | `1` | Game speed. `2` halves the run time, but `reversals/min` is then not comparable with a run at `1` |
| `--baseline <path>` | Newest other `.tsv` in the output folder | Metrics file to compare against |
| `--output-directory <path>` | `runs/ai_metrics` | Output folder |
| `--remote-port <n>` | `15702` | Port of the game's remote protocol server |

**Output:** a table with one row for each kind and script. The columns are the time to notice the
player, the engaged time, stuck time, reversals, motion switches, actions, hits and damage on the
player, the mean distance to the player, and kills. In the output folder, `<commit>.tsv` with the
table and a `<commit>` folder with a `.jsonl` recording of each run.

```sh
cargo xtask ai-metrics
cargo xtask ai-metrics --kinds FireMage --scripts still --repeats 5
cargo xtask ai-metrics --speed 2 --baseline runs/ai_metrics/765c134.tsv
```

## map-sweep

```
cargo xtask map-sweep [options]
```

Builds each map recipe with each seed. Flags enemies that try to move but stay in place, which
usually means that they spawned inside or against a wall.

| Option | Default | Description |
| --- | --- | --- |
| `--recipes <name,...>` | `Swamp,SwampWithALotOfEmptySquares,LongHallway,TreasureRoom,FireShrine,SkyTempleAscent` | Map recipes to build |
| `--seeds <n>` | `10` | Seeds for each recipe, from 1 |
| `--screenshots` | Off | Takes a screenshot of every map, not only flagged maps |
| `--output-directory <path>` | `runs/map_sweep` | Output folder |
| `--remote-port <n>` | `15702` | Port of the game's remote protocol server |

**Output:** a table with one row for each recipe: maps built, enemy and chest counts, the closest
enemy to the start, and flagged maps. In the output folder, `sweep.tsv` with one row for each map
and a screenshot of each flagged map.

```sh
cargo xtask map-sweep
cargo xtask map-sweep --recipes Swamp,FireShrine --seeds 50 --screenshots
```

# Dev keys and logs

| Key | Builds | Action |
| --- | --- | --- |
| F1 | Dev | Opens the AI test arena with a new seed |
| F2 | Native dev | Spawns 10 enemies of the selected kind at the aim reticle. Shift+F2 selects the next kind |
| F3 | Dev | Cycles the overlay: off, FPS graph, all diagnostics with sparklines |
| F4 | Dev, profiling | Logs `hitch marked at <seconds> s` |
| F5 | Dev | Pauses or resumes game time |
| F6 | Dev | Opens the next map recipe in file order with a new seed |
| F7 | Dev | When paused, runs to the next fixed step |
| F8, F9 | Dev | Halves or doubles game speed, from 1/8x to 8x |
| F10 | Native dev | Toggles player invincibility |
| F11 | Native dev | Defeats all enemies |

Dev and profiling builds also log `hitch: ...` for a frame that is at least 3 times the recent
average and slower than 60 fps.
