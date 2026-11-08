//! Development commands for the game. Run them with `cargo xtask <command>`, and pass
//! `--help` to any command for its options.

mod compare;
mod perf_report;
mod process;
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
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Sweep(args) => sweep::run(&args),
        Command::Compare(args) => compare::run(&args),
        Command::PerfReport(args) => perf_report::run(&args),
    }
}
