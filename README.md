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

# PowerShell commands

Run these from the repository root. Cargo commands run one at a time, since each takes the
build lock.

## Run

```powershell
cargo run                                   # dev build (hot reload, debug keys)
cargo run --release                         # optimized build
bevy run                                    # dev build through bevy_cli
```

## Check

```powershell
cargo check --all-targets                   # compile check
cargo clippy --all-targets                  # lints
cargo check --all-targets --no-default-features                      # release feature set
cargo check --all-targets --no-default-features --features profile   # profiling feature set
```

## Profile with Tracy

Install the Tracy v0.14.0 tools (matching `tracy-client` 0.18.5) and add them to `PATH`.
Use the `profiling` build; the dev build's timings are misleading.

```powershell
# Terminal 1: record a headless capture. Start this first, then the game.
New-Item -ItemType Directory -Force runs | Out-Null
tracy-capture -o runs\baseline.tracy -f

# Terminal 2: run the game, play the scenario, then close it.
cargo run --profile profiling --no-default-features --features profile

# Open the finished capture (or run it with no argument and click Connect for a live view).
tracy-profiler runs\baseline.tracy
```

A `profiling` build has no console, so redirect stderr to read a panic:
`.\target\profiling\baba_yaga.exe 2> panic.log`.

## Stress arena

Setting `STRESS_ARENA` to a total enemy count starts the game directly in a small arena with an invincible
player and that many enemies packed around them, split evenly across the enemy kinds. The
variable stays set in that window until you remove it.

```powershell
$env:STRESS_ARENA = "1000"

cargo run                                                                 # dev build
cargo run --profile profiling --no-default-features --features profile   # with Tracy

Remove-Item Env:STRESS_ARENA
```

`STRESS_SECONDS` (needs `STRESS_ARENA`) quits after a 5 s warm-up plus that many seconds in the
arena, and appends one summary line to `runs\stress_summary.txt`.

```powershell
$env:STRESS_SECONDS = "60"
Remove-Item Env:STRESS_SECONDS
```

Under `STRESS_ARENA` the game also skips the start screen and turns vsync off.

`scripts\stress_sweep.ps1` records one Tracy capture per run to `runs\stress\`, with the game
quitting by itself, and prints each run's summary line. It builds the profiling binary first and
needs `tracy-capture` on `PATH`.

```powershell
.\scripts\stress_sweep.ps1                                  # 100, 200, 500, 1000
.\scripts\stress_sweep.ps1 -Seconds 60 -MapSeed 7           # longer runs, another map
```

`STRESS_BUDGET_MS` (with `STRESS_SECONDS`) sets the p99 frame budget a run is flagged against, 120 fps
by default. The summary line ends its frame stats with `budget_ms` and `over_budget`.

The sweep script also takes `-Repeats` (runs per count; it prints the median of the medians),
`-CaptureSeconds` (caps each capture), `-BudgetMs` and `-Milestone <name>`, which keeps the sweep
under `runs\milestones\<date>_<name>` for later comparison.

In a dev build, F3 cycles the overlay (off, FPS graph, and a panel of every diagnostic with
sparklines). F4 marks a hitch: it writes the last 10 s of every diagnostic to
`runs\diagnostics_<time>.csv`, and under `profile` also sends a Tracy message.

`scripts\perf_report.ps1` runs `tracy-csvexport` on captures and prints ms per frame for the zones
that matter (main and render threads, fixed step, physics, sight raycasts, lights, transform
propagation, decisions, appliers), one column per capture. The fixed step is also shown per call.

```powershell
.\scripts\perf_report.ps1 runs\stress
.\scripts\perf_report.ps1 runs\baseline.tracy, runs\stress\1000_r1.tracy
```

Under `profile`, every `game/...` diagnostic is also a Tracy plot, and state changes appear as
Tracy messages (such as `AppState: StartScreen => AssetLoading` for the Start click).

## Schedule graphs

Writes `runs\schedules\update.dot` and `fixed_update.dot` from a dev build, then exits.
Rendering needs Graphviz on `PATH`.

```powershell
$env:DUMP_SCHEDULES = "1"
cargo run
Remove-Item Env:DUMP_SCHEDULES              # the variable stays set in this window until removed

dot -Tsvg runs\schedules\update.dot -o runs\schedules\update.svg
```

`runs\` is ignored by git.
