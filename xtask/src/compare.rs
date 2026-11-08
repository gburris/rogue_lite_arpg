//! Compares the summary lines stress runs write, grouped by source, enemy count, engagement and
//! player script.
//!
//! A run whose mean projectile count is far from its group's median is listed as an outlier: it
//! measured a different fight, so leave it out of an A/B.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use clap::Args;

use crate::table::{Table, optional_cell};

/// Every summary line the game writes starts with this.
const SUMMARY_PREFIX: &str = "stress summary:";

/// File a sweep or milestone folder keeps its summary lines in.
pub(super) const FOLDER_SUMMARY_FILE: &str = "summary.txt";

/// Player script of a summary line without a `player` field.
const DEFAULT_PLAYER: &str = "still";

/// Default for [`CompareArgs::outlier_fraction`].
pub(super) const DEFAULT_OUTLIER_FRACTION: f64 = 0.25;

/// Options for `compare`.
#[derive(Args)]
pub(super) struct CompareArgs {
    #[arg(
        required = true,
        help = "Summary files, or folders holding a summary.txt (a sweep or a milestone)"
    )]
    paths: Vec<PathBuf>,

    #[arg(
        long,
        default_value_t = DEFAULT_OUTLIER_FRACTION,
        help = "How far, as a fraction of its group's median, a run's projectile count may stray"
    )]
    outlier_fraction: f64,
}

/// One summary line.
struct StressRun {
    source: String,
    run_number: usize,
    enemies: u32,
    engagement: String,
    player: String,
    median_ms: Option<f64>,
    p99_ms: Option<f64>,
    max_ms: Option<f64>,
    engaged: Option<f64>,
    projectiles: Option<f64>,
    lights: Option<f64>,
}

/// The runs that share a source, engagement, player script and enemy count. Ordered for printing.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    source: String,
    engagement: String,
    player: String,
    enemies: u32,
}

pub(super) fn run(args: &CompareArgs) -> Result<()> {
    print_comparison(&args.paths, args.outlier_fraction)
}

/// Prints the group table, then any outlier runs.
pub(super) fn print_comparison(paths: &[PathBuf], outlier_fraction: f64) -> Result<()> {
    let runs = read_runs(paths)?;
    ensure!(
        !runs.is_empty(),
        "no stress summary lines found in {}",
        describe_paths(paths)
    );
    let groups = group_runs(&runs);

    println!(
        "Median of the runs' values per source, enemy count, engagement and player \
         (MaxMs is the slowest frame of any run):\n"
    );
    println!("{}", group_table(&groups));

    let outliers = find_outliers(&groups, outlier_fraction);
    if !outliers.is_empty() {
        let percent = (outlier_fraction * 100.0).round();
        println!(
            "Runs whose mean projectile count is more than {percent}% off their row's median:\n"
        );
        println!("{}", outlier_table(&outliers));
    }
    Ok(())
}

fn read_runs(paths: &[PathBuf]) -> Result<Vec<StressRun>> {
    let mut runs = Vec::new();

    for path in paths {
        let summary_file = summary_file_for(path);
        if !summary_file.is_file() {
            eprintln!("warning: no summary at {}", summary_file.display());
            continue;
        }
        runs.extend(read_summary_file(&summary_file, &source_name(path))?);
    }
    Ok(runs)
}

fn summary_file_for(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join(FOLDER_SUMMARY_FILE)
    } else {
        path.to_owned()
    }
}

/// A folder is named after itself, and a file after its name without the extension.
fn source_name(path: &Path) -> String {
    let name = if path.is_dir() {
        path.file_name()
    } else {
        path.file_stem()
    };
    name.map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn read_summary_file(file: &Path, source: &str) -> Result<Vec<StressRun>> {
    let text = fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;

    text.lines()
        .filter_map(|line| line.strip_prefix(SUMMARY_PREFIX))
        .enumerate()
        .map(|(index, fields)| {
            let run_number = index + 1;
            parse_run(fields, source, run_number)
                .with_context(|| format!("summary line {run_number} in {}", file.display()))
        })
        .collect()
}

fn parse_run(fields_text: &str, source: &str, run_number: usize) -> Result<StressRun> {
    let fields = parse_fields(fields_text);
    let required = |key: &str| {
        fields
            .get(key)
            .copied()
            .with_context(|| format!("no {key} field"))
    };
    let number = |key: &str| parse_optional_number(&fields, key);

    Ok(StressRun {
        source: source.to_owned(),
        run_number,
        enemies: required("arena")?.parse().context("arena is not a count")?,
        engagement: required("engagement")?.to_owned(),
        player: fields
            .get("player")
            .copied()
            .unwrap_or(DEFAULT_PLAYER)
            .to_owned(),
        median_ms: number("median_frame_ms")?,
        p99_ms: number("p99_frame_ms")?,
        max_ms: number("max_frame_ms")?,
        engaged: number("mean_game/ai/engaged")?,
        projectiles: number("mean_game/projectiles")?,
        lights: number("mean_game/lights")?,
    })
}

/// Splits `key=value key=value ...` into its fields.
fn parse_fields(fields_text: &str) -> HashMap<&str, &str> {
    fields_text
        .split_whitespace()
        .filter_map(|field| field.split_once('='))
        .collect()
}

/// The field as a number, or `None` when it is missing or `-`.
fn parse_optional_number(fields: &HashMap<&str, &str>, key: &str) -> Result<Option<f64>> {
    match fields.get(key) {
        None | Some(&"-") => Ok(None),
        Some(text) => {
            let number = text
                .parse()
                .with_context(|| format!("{key}={text} is not a number"))?;
            Ok(Some(number))
        }
    }
}

fn group_runs(runs: &[StressRun]) -> BTreeMap<GroupKey, Vec<&StressRun>> {
    let mut groups = BTreeMap::<GroupKey, Vec<&StressRun>>::new();

    for run in runs {
        let key = GroupKey {
            source: run.source.clone(),
            engagement: run.engagement.clone(),
            player: run.player.clone(),
            enemies: run.enemies,
        };
        groups.entry(key).or_default().push(run);
    }
    groups
}

fn group_table(groups: &BTreeMap<GroupKey, Vec<&StressRun>>) -> Table {
    let mut table = Table::new([
        "Source",
        "Enemies",
        "Engagement",
        "Player",
        "Runs",
        "MedianMs",
        "P99Ms",
        "MaxMs",
        "Engaged",
        "Projectiles",
        "Lights",
    ]);

    for (key, runs) in groups {
        let median_of = |value: fn(&StressRun) -> Option<f64>| {
            optional_cell(lower_median(runs.iter().filter_map(|run| value(run))))
        };
        let slowest_frame = runs.iter().filter_map(|run| run.max_ms).reduce(f64::max);

        table.add_row(vec![
            key.source.clone(),
            key.enemies.to_string(),
            key.engagement.clone(),
            key.player.clone(),
            runs.len().to_string(),
            median_of(|run| run.median_ms),
            median_of(|run| run.p99_ms),
            optional_cell(slowest_frame),
            median_of(|run| run.engaged),
            median_of(|run| run.projectiles),
            median_of(|run| run.lights),
        ]);
    }
    table
}

/// Runs whose projectile count strays more than `outlier_fraction` of their group's median.
fn find_outliers<'run>(
    groups: &BTreeMap<GroupKey, Vec<&'run StressRun>>,
    outlier_fraction: f64,
) -> Vec<&'run StressRun> {
    let mut outliers = Vec::new();

    for runs in groups.values() {
        let Some(median) = lower_median(runs.iter().filter_map(|run| run.projectiles)) else {
            continue;
        };
        if median < 1.0 {
            continue;
        }

        let allowed_deviation = outlier_fraction * median;
        let strays = |run: &&&StressRun| {
            run.projectiles
                .is_some_and(|projectiles| (projectiles - median).abs() > allowed_deviation)
        };
        outliers.extend(runs.iter().filter(strays).copied());
    }
    outliers
}

fn outlier_table(outliers: &[&StressRun]) -> Table {
    let mut table = Table::new([
        "Source",
        "Run",
        "Enemies",
        "Engagement",
        "Player",
        "MedianMs",
        "P99Ms",
        "Projectiles",
        "Engaged",
    ]);

    for run in outliers {
        table.add_row(vec![
            run.source.clone(),
            run.run_number.to_string(),
            run.enemies.to_string(),
            run.engagement.clone(),
            run.player.clone(),
            optional_cell(run.median_ms),
            optional_cell(run.p99_ms),
            optional_cell(run.projectiles),
            optional_cell(run.engaged),
        ]);
    }
    table
}

/// The lower of the two middle values, matching the game's 50th percentile.
fn lower_median(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut sorted = values.collect::<Vec<_>>();
    if sorted.is_empty() {
        return None;
    }

    sorted.sort_by(f64::total_cmp);
    Some(sorted[(sorted.len() - 1) / 2])
}

fn describe_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
