![image](https://github.com/user-attachments/assets/8e19637c-eb1f-4170-b202-663c1434e073)

# Building

## Native
```bash
# Prerequisites
rustup update

# Build
cargo run           # dev build
cargo run --release # optimized build
```

## Web Assembly

```bash
# Prerequisites
rustup update
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build
wasm-pack build --target web --out-dir dist
```

# Commands

Examples use PowerShell. Run these from the repository root. Run Cargo commands one at a time, since each takes the build
lock. Environment variables stay set in the PowerShell window until you remove them with
`Remove-Item Env:<NAME>`.

Each command below lists its usage, then options, output and examples where it has them.

## Run

```
cargo run [--release]
bevy run
```

Builds and starts the game.

| Command | Build |
| --- | --- |
| `cargo run` | Dev build: hot reload, dev keys, diagnostics overlay |
| `cargo run --release` | Optimized build |
| `bevy run` | Dev build through `bevy_cli` |

## Check

```
cargo check --all-targets [--no-default-features [--features profile]]
cargo clippy --workspace --all-targets
```

Compile checks and lints. No game window opens.

| Command | Checks |
| --- | --- |
| `cargo check --all-targets` | Dev feature set |
| `cargo check --all-targets --no-default-features` | Release feature set |
| `cargo check --all-targets --no-default-features --features profile` | Profiling feature set |
| `cargo clippy --workspace --all-targets` | Lints, for the game and `xtask` |

## Profile with Tracy

```
tracy-capture -o <file.tracy> -f
cargo run --profile profiling --no-default-features --features profile
tracy-profiler [<file.tracy>]
```

Records a Tracy capture of the `profiling` build, then opens it. Use the `profiling` build for
timings; the dev build's are misleading.

**Requires:** the Tracy v0.14.0 tools (matching `tracy-client` 0.18.5) on `PATH`.

**Output:** the capture file. In a capture, every `game/...` diagnostic is a plot, and state
changes and log lines are messages (such as `AppState: StartScreen => AssetLoading` for the
Start click, or `hitch: ...`).

**Examples:**

```powershell
# Terminal 1: start the capture first, then the game.
New-Item -ItemType Directory -Force runs | Out-Null
tracy-capture -o runs\baseline.tracy -f

# Terminal 2: run the game, play the scenario, then close it.
cargo run --profile profiling --no-default-features --features profile

# Open the finished capture, or run it with no argument and click Connect for a live view.
tracy-profiler runs\baseline.tracy
```

**Notes:** a `profiling` build has no console, so redirect stderr to read a panic:
`.\target\profiling\baba_yaga.exe 2> panic.log`.

## Stress arena

```
cargo run [--profile profiling --no-default-features --features profile]
          -- --stress <enemies> [--seconds <seconds>] [--engagement all|sight|none]
```

Starts the game directly in a small arena: an invincible player with that many enemies packed
around them, split evenly across the enemy kinds. The start screen is skipped and vsync is off.
The flags exist in dev and profiling builds.

**Options:**

| Option | Default | Description |
| --- | --- | --- |
| `--stress` | none (normal game) | Total enemies |
| `--seconds` | none (runs until closed) | Quits after a 5 s warm-up plus this many seconds, and writes the summary line |
| `--engagement` | `all` | How enemies find the player (see below) |

`MAP_SEED` (an environment variable) fixes the map layout and enemy placement, in the arena or
not.

| `--engagement` | Enemies | Use it for |
| --- | --- | --- |
| `all` | Target the player from spawn, until the player hits them | Repeatable A/B runs: every run fights the same way |
| `sight` | Engage on sight, as in normal play | Runs land in a heavy or a calm fight by chance |
| `none` | Are blind and only wander | The cost of the crowd without combat |

**Output:** with `--seconds`, one summary line in the log, also appended to
`runs\stress_summary.txt`:

- the settings: `arena`, `engagement`, `seconds`;
- `startup_seconds`: first frame to arena, covering asset loading and map spawn;
- `frames`, `median_frame_ms`, `p99_frame_ms` and `max_frame_ms`;
- the mean of every `game/...` diagnostic, such as `mean_game/ai/engaged` and
  `mean_game/projectiles`.

**Examples:**

```powershell
cargo run -- --stress 1000                                     # dev build, until closed
cargo run -- --stress 1000 --seconds 60 --engagement none      # timed run, crowd only
cargo run --profile profiling --no-default-features --features profile -- --stress 1000
```

## xtask

```
cargo run -p xtask -- <sweep|compare|perf-report> [options]
```

Development commands written in Rust, so they run anywhere `cargo` does. Run them from anywhere
in the repository. `--help` after a command lists its options.

### sweep

```
cargo run -p xtask -- sweep [--counts <n,n,...>] [--repeats <n>] [--engagement <all,sight,none>]
                            [--seconds <n>] [--capture-seconds <n>] [--map-seed <n>]
                            [--milestone <name>] [--output-directory <path>]
```

Builds the profiling binary once, then records one Tracy capture per engagement, enemy count and
repeat in the stress arena. Each run quits by itself.

**Requires:** `tracy-capture` on `PATH`.

**Options:**

| Option | Default | Description |
| --- | --- | --- |
| `--counts` | `100,200,500,1000` | Total enemies per run |
| `--repeats` | `1` | Runs per count and engagement. Use 2-3; the comparison prints the median |
| `--engagement` | `all` | One or more of `all`, `sight` and `none` |
| `--seconds` | `30` | Measured seconds per run, after the 5 s warm-up |
| `--capture-seconds` | none (whole run) | Stops each capture early. Use 15-20 before exporting events, since a 30 s capture exports about 2.5 GB |
| `--map-seed` | `1` | Map layout and enemy placement |
| `--milestone` | none | Writes to `runs\milestones\<date>_<name>`, which no later sweep overwrites |
| `--output-directory` | `runs\stress` | Where captures go without `--milestone`. A rerun overwrites them |

**Output:** `<engagement>_<count>_r<repeat>.tracy` captures and a `summary.txt` of the sweep's
summary lines, both in the output folder. It then prints the runs through `compare`.

**Examples:**

```powershell
cargo run -p xtask -- sweep
cargo run -p xtask -- sweep --counts 500,1000 --repeats 3 --capture-seconds 20
cargo run -p xtask -- sweep --counts 1000 --engagement all,none --milestone substeps
```

### compare

```
cargo run -p xtask -- compare <path>... [--outlier-fraction <fraction>]
```

Compares stress summary lines, one row per source, enemy count and engagement.

**Options:**

| Option | Default | Description |
| --- | --- | --- |
| `<path>...` | required | Summary files, or folders holding a `summary.txt` (a sweep or a milestone) |
| `--outlier-fraction` | `0.25` | How far a run's projectile count may be from its row's median before it is listed |

**Output:** a table with the median of the runs' median and p99 frame ms, the slowest frame, and
the engaged brains, projectiles and lights. Then a list of outlier runs, whose projectile count
shows they measured a different fight.

**Examples:**

```powershell
cargo run -p xtask -- compare runs\stress
cargo run -p xtask -- compare runs\milestones\2026-11-01_before runs\milestones\2026-11-03_after
```

### perf-report

```
cargo run -p xtask -- perf-report <path>...
```

Runs `tracy-csvexport` on captures and prints ms per frame for the zones that matter: main and
render threads, fixed step, physics, sight raycasts, lights, transform propagation, decisions and
appliers. The fixed step is also shown per call, since it only runs on some frames.

**Requires:** `tracy-csvexport` on `PATH`.

**Options:**

| Option | Default | Description |
| --- | --- | --- |
| `<path>...` | required | Folders of `.tracy` captures, or `.tracy` files |

**Output:** one row per zone, one column per capture. Frame counts include warm-up and loading,
so compare captures made the same way.

**Examples:**

```powershell
cargo run -p xtask -- perf-report runs\stress
cargo run -p xtask -- perf-report runs\baseline.tracy runs\stress\all_1000_r1.tracy
```

## Schedule graphs

```
$env:DUMP_SCHEDULES = "1"
cargo run
dot -Tsvg <file.dot> -o <file.svg>
```

Writes the schedule graphs from a dev build, then exits.

**Requires:** Graphviz on `PATH` to render the graphs.

**Output:** `runs\schedules\update.dot` and `runs\schedules\fixed_update.dot`.

**Examples:**

```powershell
$env:DUMP_SCHEDULES = "1"
cargo run
Remove-Item Env:DUMP_SCHEDULES

dot -Tsvg runs\schedules\update.dot -o runs\schedules\update.svg
```

## Dev keys and logs

| Key | Builds | Action |
| --- | --- | --- |
| F1 | dev | Opens the AI test arena, with a fresh seed |
| F3 | dev | Cycles the overlay: off, FPS graph, and a panel of every diagnostic with sparklines |
| F4 | dev, profiling | Logs `hitch marked at <seconds> s` |
| F6 | dev | Opens the next map recipe in file order, with a fresh seed |

Dev and profiling builds also log `hitch: ...` for every frame at least 3 times the recent
average and slower than 60 fps.

`runs\` is ignored by git.
