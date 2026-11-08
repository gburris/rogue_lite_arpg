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
cargo xtask <sweep|compare|perf-report> [options]
```

Development commands written in Rust. Run them from any folder in the repository. Add `--help`
to see all options.

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
