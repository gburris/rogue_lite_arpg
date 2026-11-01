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

Add `profile_memory` to `--features` to track allocations, which adds noticeable overhead.
A `profiling` build has no console, so redirect stderr to read a panic:
`.\target\profiling\baba_yaga.exe 2> panic.log`.

## Stress arena

Setting `STRESS_ARENA` makes Start load a 200 x 200 open arena with an invincible player and
the listed enemy counts. Kinds are `warrior`, `ice_mage`, `fire_mage` and `bat`; kinds you leave
out spawn none. The variable stays set in that window until you remove it.

```powershell
$env:STRESS_ARENA = "warrior=250,ice_mage=250,fire_mage=250,bat=250"

cargo run                                                                 # dev build
cargo run --profile profiling --no-default-features --features profile   # with Tracy

Remove-Item Env:STRESS_ARENA
```

`STRESS_SECONDS` (needs `STRESS_ARENA`) quits after a 5 s warm-up plus that many seconds in the
arena, and appends one summary line to `runs\stress_summary.txt`. In a dev build, F2 rebuilds the
arena at the next total of 100, 200, 500 and 1000 enemies, split evenly across the kinds.

```powershell
$env:STRESS_SECONDS = "60"
Remove-Item Env:STRESS_SECONDS
```

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
