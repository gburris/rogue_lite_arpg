//! Records Tracy captures of the profiling build in the stress arena, one per engagement, enemy
//! count and repeat, then compares the runs' summary lines.
//!
//! Each run starts `tracy-capture`, then the game with `--seconds` so it quits by itself.
//! The game appends its summary line to `runs/stress_summary.txt`; the sweep copies its own
//! lines to a `summary.txt` beside the captures, and each run's log to `<run>.log`.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{env, fs, slice, thread};

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};

use crate::compare::{DEFAULT_OUTLIER_FRACTION, FOLDER_SUMMARY_FILE, print_comparison};
use crate::process::{ChildProcess, require_on_path, run_to_completion};

const CAPTURE_TOOL: &str = "tracy-capture";
const GAME_BINARY: &str = "baba_yaga";
const PROFILING_PROFILE: &str = "profiling";
const GAME_SUMMARY_PATH: &str = "runs/stress_summary.txt";

/// Time `tracy-capture` gets to start listening before the game connects.
const CAPTURE_STARTUP: Duration = Duration::from_secs(2);

/// Slack on top of the measured seconds for warm-up, startup and shutdown.
const GAME_EXIT_SLACK: Duration = Duration::from_secs(120);

/// Time `tracy-capture` gets to save once the game has quit.
const CAPTURE_SAVE_TIMEOUT: Duration = Duration::from_secs(60);

/// Options for `sweep`.
#[derive(Args)]
pub(super) struct SweepArgs {
    #[arg(
        long,
        value_delimiter = ',',
        default_values_t = [100, 200, 500, 1000],
        help = "Total enemies per run, split evenly across the enemy kinds"
    )]
    counts: Vec<u32>,

    #[arg(
        long,
        default_value_t = 1,
        value_parser = clap::value_parser!(u32).range(1..=100),
        help = "Runs per count and engagement; use 2-3, since the comparison prints the median"
    )]
    repeats: u32,

    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = [Engagement::All],
        help = "How enemies find the player (see the stress arena in the README)"
    )]
    engagement: Vec<Engagement>,

    #[arg(
        long,
        default_value_t = 30,
        help = "Measured seconds per run, after the 5 s warm-up"
    )]
    seconds: u64,

    #[arg(
        long,
        help = "Stops each capture after this many seconds; use 15-20 before exporting events"
    )]
    capture_seconds: Option<u64>,

    #[arg(long, default_value_t = 1, help = "Map layout and enemy placement")]
    map_seed: u64,

    #[arg(
        long,
        help = "Writes to runs/milestones/<date>_<name>, which no later sweep overwrites"
    )]
    milestone: Option<String>,

    #[arg(
        long,
        default_value = "runs/stress",
        help = "Where captures go without --milestone; a rerun overwrites them"
    )]
    output_directory: PathBuf,
}

/// The values the game's `--engagement` takes. Keep in step with `Engagement` in
/// `src/dev/stress.rs`.
#[derive(ValueEnum, Clone, Copy, Debug)]
enum Engagement {
    All,
    Sight,
    #[value(name = "none")]
    Blind,
}

/// One game run of the sweep.
#[derive(Clone, Copy, Debug)]
struct PlannedRun {
    engagement: Engagement,
    enemies: u32,
    repeat: u32,
}

pub(super) fn run(args: &SweepArgs) -> Result<()> {
    enter_repository_root()?;
    require_on_path(CAPTURE_TOOL)?;

    build_profiling_game()?;
    let game = profiling_game_path();
    let output_directory = output_directory(args);
    fs::create_dir_all(&output_directory)
        .with_context(|| format!("creating {}", output_directory.display()))?;

    let summary_lines_before = read_game_summary_lines()?.len();
    for planned_run in planned_runs(args) {
        record_run(planned_run, args, &game, &output_directory)?;
    }
    let sweep_lines = read_game_summary_lines()?.split_off(summary_lines_before);

    println!("\nDone.");
    if sweep_lines.is_empty() {
        eprintln!("warning: no new summary lines; the game runs probably failed to start");
        return Ok(());
    }
    save_sweep_summary(&sweep_lines, &output_directory)?;

    println!();
    print_comparison(slice::from_ref(&output_directory), DEFAULT_OUTLIER_FRACTION)?;
    if args.milestone.is_some() {
        println!("milestone saved in {}", output_directory.display());
    }
    Ok(())
}

/// Moves to the repository root, so relative paths and the game's assets resolve from anywhere.
fn enter_repository_root() -> Result<()> {
    let root = repository_root();
    env::set_current_dir(&root).with_context(|| format!("entering {}", root.display()))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in a folder of the repository")
        .to_owned()
}

/// Builds with the `cargo` that launched `xtask`, falling back to the one on `PATH`.
fn build_profiling_game() -> Result<()> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    run_to_completion(
        "cargo build",
        Command::new(cargo).args([
            "build",
            "--profile",
            PROFILING_PROFILE,
            "--no-default-features",
            "--features",
            "tracy",
        ]),
    )
}

fn profiling_game_path() -> PathBuf {
    let target_directory = env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| repository_root().join("target"), PathBuf::from);
    let executable = format!("{GAME_BINARY}{}", env::consts::EXE_SUFFIX);

    target_directory.join(PROFILING_PROFILE).join(executable)
}

fn output_directory(args: &SweepArgs) -> PathBuf {
    match &args.milestone {
        Some(name) => {
            let today = jiff::Zoned::now().date();
            ["runs", "milestones", &format!("{today}_{name}")]
                .iter()
                .collect()
        }
        None => args.output_directory.clone(),
    }
}

fn planned_runs(args: &SweepArgs) -> Vec<PlannedRun> {
    let mut runs = Vec::new();

    for &engagement in &args.engagement {
        for &enemies in &args.counts {
            for repeat in 1..=args.repeats {
                runs.push(PlannedRun {
                    engagement,
                    enemies,
                    repeat,
                });
            }
        }
    }
    runs
}

fn record_run(
    planned_run: PlannedRun,
    args: &SweepArgs,
    game: &Path,
    output_directory: &Path,
) -> Result<()> {
    let capture_file = output_directory.join(format!("{}.tracy", planned_run.file_stem()));
    let game_log_file = output_directory.join(format!("{}.log", planned_run.file_stem()));
    println!(
        "== {} enemies, engagement {}, repeat {} -> {}",
        planned_run.enemies,
        planned_run.engagement.name(),
        planned_run.repeat,
        capture_file.display()
    );

    let capture = start_capture(&capture_file, args.capture_seconds)?;
    thread::sleep(CAPTURE_STARTUP);
    let game = start_game(game, planned_run, args, &game_log_file)?;

    game.wait_or_kill(Duration::from_secs(args.seconds) + GAME_EXIT_SLACK)?;
    capture.wait_or_kill(CAPTURE_SAVE_TIMEOUT)
}

fn start_capture(capture_file: &Path, capture_seconds: Option<u64>) -> Result<ChildProcess> {
    let mut command = Command::new(CAPTURE_TOOL);
    command.arg("-o").arg(capture_file).arg("-f");
    if let Some(seconds) = capture_seconds {
        command.args(["-s", &seconds.to_string()]);
    }

    ChildProcess::spawn(CAPTURE_TOOL, &mut command)
}

fn start_game(
    game: &Path,
    planned_run: PlannedRun,
    args: &SweepArgs,
    log_file: &Path,
) -> Result<ChildProcess> {
    let log = File::create(log_file).with_context(|| format!("creating {}", log_file.display()))?;

    let mut command = Command::new(game);
    command
        .stdout(log.try_clone()?)
        .stderr(log)
        .env("BEVY_ASSET_ROOT", repository_root())
        .env("MAP_SEED", args.map_seed.to_string())
        .args(["--stress", &planned_run.enemies.to_string()])
        .args(["--seconds", &args.seconds.to_string()])
        .args(["--engagement", planned_run.engagement.name()]);

    ChildProcess::spawn("the game", &mut command)
}

/// Every line of the game's summary file, or none when no run has written it yet.
fn read_game_summary_lines() -> Result<Vec<String>> {
    if !Path::new(GAME_SUMMARY_PATH).exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(GAME_SUMMARY_PATH)
        .with_context(|| format!("reading {GAME_SUMMARY_PATH}"))?;
    Ok(text.lines().map(str::to_owned).collect())
}

fn save_sweep_summary(sweep_lines: &[String], output_directory: &Path) -> Result<()> {
    for line in sweep_lines {
        println!("{line}");
    }

    let summary_file = output_directory.join(FOLDER_SUMMARY_FILE);
    let text = sweep_lines.join("\n") + "\n";
    fs::write(&summary_file, text).with_context(|| format!("writing {}", summary_file.display()))
}

impl PlannedRun {
    /// The name shared by the run's capture and log, such as `all_1000_r1`.
    fn file_stem(self) -> String {
        format!(
            "{}_{}_r{}",
            self.engagement.name(),
            self.enemies,
            self.repeat
        )
    }
}

impl Engagement {
    /// The name the game's `--engagement` takes.
    fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Sight => "sight",
            Self::Blind => "none",
        }
    }
}
