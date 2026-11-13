//! Development commands for the game. Run them with `cargo xtask <command>`, and pass
//! `--help` to any command for its options.

mod ai_metrics;
mod compare;
mod map_sweep;
mod perf_report;
mod playthrough;
mod process;
mod remote;
mod sweep;
mod table;

use clap::{Parser, Subcommand};

/// The command line: one subcommand per tool.
#[derive(Parser)]
#[command(about = "Stress sweeps and Tracy reports for the game")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The tools `xtask` runs.
#[derive(Subcommand)]
enum Command {
    /// Records Tracy captures of the profiling build in the stress arena, then compares them.
    Sweep(sweep::SweepArgs),
    /// Compares stress summary lines, one row per source, enemy count, engagement and player.
    Compare(compare::CompareArgs),
    /// Prints ms per frame for the zones that matter, one column per Tracy capture.
    PerfReport(perf_report::PerfReportArgs),
    /// Plays the dev build from the start screen through a fight, death and restart over the
    /// remote protocol, screenshotting each step.
    Playthrough(playthrough::PlaythroughArgs),
    /// Measures enemy movement and fighting in the AI test room and compares it with an earlier
    /// run.
    AiMetrics(ai_metrics::AiMetricsArgs),
    /// Builds map recipes over many seeds and flags enemies that spawn stuck.
    MapSweep(map_sweep::MapSweepArgs),
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Sweep(args) => sweep::run(&args),
        Command::Compare(args) => compare::run(&args),
        Command::PerfReport(args) => perf_report::run(&args),
        Command::Playthrough(args) => playthrough::run(&args),
        Command::AiMetrics(args) => ai_metrics::run(&args),
        Command::MapSweep(args) => map_sweep::run(&args),
    }
}
