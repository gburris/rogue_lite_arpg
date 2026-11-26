use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result, ensure};
use clap::{Args, ValueEnum};

use crate::ai_metrics::commit_name;
use crate::metrics_comparison::BASELINE_REPEATS_FOR_BAND;
use crate::remote::RemoteArgs;
use crate::sweep::enter_repository_root;
use crate::table::Table;

/// Where walled-room `ai-metrics` runs go, apart from the open-room runs.
const WALL_ROOM_OUTPUT: &str = "runs/ai_metrics_wall";

/// Options for `regression`.
#[derive(Args)]
pub(super) struct RegressionArgs {
    #[command(flatten)]
    remote: RemoteArgs,

    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = GateRow::ALL,
        help = "Gate rows to run, in this order"
    )]
    rows: Vec<GateRow>,

    #[arg(
        long,
        help = format!(
            "Open-room `ai-metrics` baseline; defaults to the newest other run in its folder. \
             Record it with --repeats {BASELINE_REPEATS_FOR_BAND}, so its noise band can fail \
             the row"
        )
    )]
    ai_metrics_baseline: Option<PathBuf>,

    #[arg(
        long,
        help = format!(
            "Walled-room `ai-metrics` baseline; defaults to the newest other run in \
             {WALL_ROOM_OUTPUT}. Record it with --repeats {BASELINE_REPEATS_FOR_BAND}"
        )
    )]
    wall_baseline: Option<PathBuf>,

    #[arg(
        long,
        default_value = "runs/regression",
        help = "Where `<commit>/summary.txt` and each row's output go"
    )]
    output_directory: PathBuf,
}

/// One row of the regression gate: a tool run with the options the gate gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum GateRow {
    Playthrough,
    Checks,
    AiMetrics,
    AiMetricsWall,
    MapSweep,
}

impl GateRow {
    const ALL: [Self; 5] = [
        Self::Playthrough,
        Self::Checks,
        Self::AiMetrics,
        Self::AiMetricsWall,
        Self::MapSweep,
    ];

    /// The name `--rows` takes it by.
    fn name(self) -> String {
        self.to_possible_value()
            .expect("no gate row is skipped from --rows")
            .get_name()
            .to_owned()
    }

    /// The `xtask` command line that runs this row.
    fn build_arguments(self, args: &RegressionArgs) -> Vec<String> {
        let mut arguments: Vec<String> = match self {
            Self::Playthrough => vec!["playthrough".to_owned()],
            Self::Checks => vec!["checks".to_owned()],
            Self::AiMetrics => vec!["ai-metrics".to_owned()],
            Self::AiMetricsWall => vec![
                "ai-metrics".to_owned(),
                "--room".to_owned(),
                "AiTestRoomWall".to_owned(),
                "--output-directory".to_owned(),
                WALL_ROOM_OUTPUT.to_owned(),
            ],
            Self::MapSweep => vec!["map-sweep".to_owned()],
        };
        let baseline = match self {
            Self::AiMetrics => args.ai_metrics_baseline.as_ref(),
            Self::AiMetricsWall => args.wall_baseline.as_ref(),
            Self::Playthrough | Self::Checks | Self::MapSweep => None,
        };
        if let Some(baseline) = baseline {
            arguments.extend(["--baseline".to_owned(), baseline.display().to_string()]);
        }
        arguments.extend(args.remote.to_arguments());
        arguments
    }
}

const _: () = match GateRow::Playthrough {
    GateRow::Playthrough
    | GateRow::Checks
    | GateRow::AiMetrics
    | GateRow::AiMetricsWall
    | GateRow::MapSweep => {}
};

/// Runs each gate row as its own `xtask` command, one after another, saving each one's output,
/// then prints and saves a summary with one pass or fail per row. A row that fails, panics or
/// can't start doesn't stop the rows after it; the gate fails at the end.
pub(super) fn run(args: &RegressionArgs) -> Result<()> {
    enter_repository_root()?;
    let commit_folder = args.output_directory.join(commit_name()?);
    fs::create_dir_all(&commit_folder)
        .with_context(|| format!("creating {}", commit_folder.display()))?;
    let xtask = env::current_exe().context("finding the xtask executable")?;

    let mut table = Table::new(["row", "result", "minutes", "notes"]);
    let mut failed_rows = 0;
    for &row in &args.rows {
        let name = row.name();
        println!("== {name}");
        let output_file = commit_folder.join(format!("{name}.txt"));
        let started = Instant::now();
        let outcome = run_row(&xtask, &row.build_arguments(args), &output_file);
        let (result, notes) = match outcome {
            Ok(true) => ("ok", format!("output in {}", output_file.display())),
            Ok(false) => ("FAILED", find_failure_note(&output_file)),
            Err(error) => ("FAILED", format!("{error:#}")),
        };
        if result != "ok" {
            failed_rows += 1;
        }
        table.add_row(vec![
            name,
            result.to_owned(),
            format!("{:.1}", started.elapsed().as_secs_f64() / 60.0),
            notes,
        ]);
    }

    let summary = table.to_string();
    let summary_file = commit_folder.join("summary.txt");
    fs::write(&summary_file, &summary)
        .with_context(|| format!("writing {}", summary_file.display()))?;
    println!("\n{summary}");
    println!("saved {}", summary_file.display());
    ensure!(failed_rows == 0, "gate rows failed: {failed_rows}");
    Ok(())
}

fn run_row(xtask: &Path, arguments: &[String], output_file: &Path) -> Result<bool> {
    let row_output =
        File::create(output_file).with_context(|| format!("creating {}", output_file.display()))?;
    let status = Command::new(xtask)
        .args(arguments)
        .stdout(
            row_output
                .try_clone()
                .context("sharing the row's output file")?,
        )
        .stderr(row_output)
        .status()
        .with_context(|| format!("running xtask {}", arguments.join(" ")))?;
    Ok(status.success())
}

/// Why a failed row failed, from the error or panic its output ends with, pointing at the output
/// for the lines before it.
fn find_failure_note(output_file: &Path) -> String {
    let text = fs::read(output_file).map_or_else(
        |_| String::new(),
        |bytes| String::from_utf8_lossy(&bytes).into_owned(),
    );
    let lines: Vec<&str> = text.lines().collect();
    let error = lines
        .iter()
        .rposition(|line| line.starts_with("Error: "))
        .map(|start| {
            // anyhow prints each cause under `Caused by:` as an indented line, numbered when
            // there are several.
            let causes = lines[start + 1..]
                .iter()
                .filter(|line| line.starts_with(char::is_whitespace))
                .map(|line| {
                    let cause = line.trim();
                    cause
                        .split_once(": ")
                        .filter(|(number, _)| number.parse::<u32>().is_ok())
                        .map_or(cause, |(_, numbered_cause)| numbered_cause)
                });
            std::iter::once(&lines[start]["Error: ".len()..])
                .chain(causes)
                .collect::<Vec<_>>()
                .join(": ")
        });
    let panic = || {
        let at = lines
            .iter()
            .rposition(|line| line.contains("panicked at"))?;
        Some(format!("panicked: {}", lines.get(at + 1).unwrap_or(&"")))
    };
    let failure = error
        .or_else(panic)
        .unwrap_or_else(|| "no error line".to_owned());
    format!("{failure}; see {}", output_file.display())
}
