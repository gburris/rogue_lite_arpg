//! Prints ms per frame, or per call, for the zones that matter to this game, one column per
//! Tracy capture.
//!
//! Frames are counted from the `main app` zone, which includes the warm-up and loading, so
//! compare captures made the same way. The fixed step only runs on some frames, so it is also
//! shown per call.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{Context, Result, bail, ensure};
use clap::Args;

use crate::process::require_on_path;
use crate::table::{MISSING, Table, with_thousands_separators};

const CSV_EXPORT: &str = "tracy-csvexport";
const CSV_SEPARATOR: &str = ";";
const CAPTURE_EXTENSION: &str = "tracy";
const FRAME_ZONE: &str = "main app";
const NANOSECONDS_PER_MILLISECOND: f64 = 1e6;

/// Options for `perf-report`.
#[derive(Args)]
pub(super) struct PerfReportArgs {
    #[arg(required = true, help = "Folders of .tracy captures, or .tracy files")]
    paths: Vec<PathBuf>,
}

/// A row of the report: one or more Tracy zones, summed.
struct Zone {
    label: &'static str,
    names: ZoneNames,
    basis: Basis,
}

/// Which Tracy zone names a [`Zone`] sums.
enum ZoneNames {
    Exactly(String),
    BrainAppliers,
}

/// What a [`Zone`]'s total time is divided by.
#[derive(Clone, Copy, Debug)]
enum Basis {
    PerFrame,
    PerCall,
}

/// Time and calls of one zone name, summed over the rows `tracy-csvexport` prints for it.
#[derive(Clone, Copy, Debug, Default)]
struct ZoneTotals {
    nanoseconds: f64,
    calls: f64,
}

/// One capture's column of the report.
struct CaptureColumn {
    name: String,
    frames: f64,
    totals_by_zone_name: HashMap<String, ZoneTotals>,
}

pub(super) fn run(args: &PerfReportArgs) -> Result<()> {
    require_on_path(CSV_EXPORT)?;
    let captures = find_captures(&args.paths)?;
    let column_names = column_names(&captures);

    let mut columns = Vec::new();
    for (capture, name) in captures.iter().zip(column_names) {
        println!("reading {}", file_name(capture));
        columns.push(read_capture_column(capture, name)?);
    }

    println!("\n{}", report_table(&zones(), &columns));
    Ok(())
}

fn zones() -> Vec<Zone> {
    vec![
        Zone::per_frame("main app", ZoneNames::Exactly(FRAME_ZONE.to_owned())),
        Zone::per_frame("render app", sub_app("RenderApp")),
        Zone::per_call("FixedMain", schedule("FixedMain")),
        Zone::per_frame("FixedMain (per frame)", schedule("FixedMain")),
        Zone::per_frame("physics", schedule("PhysicsSchedule")),
        Zone::per_frame("substeps", schedule("SubstepSchedule")),
        Zone::per_frame("sight raycasts", system("avian2d::spatial_query::raycast")),
        Zone::per_frame(
            "light buffers",
            system(concat!(
                "bevy_lit::light::render::prepare_light2d_buffers",
                "<bevy_lit::light::point_light::PointLight2d>"
            )),
        ),
        Zone::per_frame(
            "transform propagation",
            system("bevy_transform::systems::parallel::propagate_parent_transforms"),
        ),
        Zone::per_frame(
            "decide_behavior",
            system("baba_yaga::character::brain::decide::decide_behavior"),
        ),
        Zone::per_frame("appliers", ZoneNames::BrainAppliers),
    ]
}

fn sub_app(name: &str) -> ZoneNames {
    ZoneNames::Exactly(format!("sub app{{name={name}}}"))
}

fn schedule(name: &str) -> ZoneNames {
    ZoneNames::Exactly(format!("schedule{{name={name}}}"))
}

fn system(path: &str) -> ZoneNames {
    ZoneNames::Exactly(system_zone_name(path))
}

fn system_zone_name(path: &str) -> String {
    format!("system{{name=\"{path}\"}}")
}

impl Zone {
    fn per_frame(label: &'static str, names: ZoneNames) -> Self {
        Self {
            label,
            names,
            basis: Basis::PerFrame,
        }
    }

    fn per_call(label: &'static str, names: ZoneNames) -> Self {
        Self {
            label,
            names,
            basis: Basis::PerCall,
        }
    }

    fn unit(&self) -> &'static str {
        match self.basis {
            Basis::PerFrame => "ms/frame",
            Basis::PerCall => "ms/call",
        }
    }

    /// Milliseconds per frame or per call in `column`, or `None` when there is nothing to
    /// divide by.
    fn milliseconds(&self, column: &CaptureColumn) -> Option<f64> {
        let totals = column
            .totals_by_zone_name
            .iter()
            .filter(|(name, _)| self.names.matches(name))
            .fold(ZoneTotals::default(), |sum, (_, totals)| sum.plus(*totals));

        let divisor = match self.basis {
            Basis::PerFrame => column.frames,
            Basis::PerCall => totals.calls,
        };
        (divisor > 0.0).then(|| totals.nanoseconds / divisor / NANOSECONDS_PER_MILLISECOND)
    }
}

impl ZoneNames {
    fn matches(&self, zone_name: &str) -> bool {
        match self {
            Self::Exactly(name) => zone_name == name,
            Self::BrainAppliers => is_brain_applier(zone_name),
        }
    }
}

/// The equipment use applier, or any `apply_*` system in a `brain::motion` module.
fn is_brain_applier(zone_name: &str) -> bool {
    const EQUIPMENT_APPLIER: &str =
        "baba_yaga::character::brain::action::equipment_use::apply_equipment_use_requests";
    const MOTION_MODULES: &str = "baba_yaga::character::brain::motion::";

    let Some(path) = zone_name
        .strip_prefix("system{name=\"")
        .and_then(|rest| rest.strip_suffix("\"}"))
    else {
        return false;
    };
    if path == EQUIPMENT_APPLIER {
        return true;
    }

    let Some((module, system)) = path
        .strip_prefix(MOTION_MODULES)
        .and_then(|rest| rest.split_once("::"))
    else {
        return false;
    };
    let applied = system.strip_prefix("apply_");

    is_snake_case(module) && applied.is_some_and(is_snake_case)
}

fn is_snake_case(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '_')
}

impl ZoneTotals {
    fn plus(self, other: Self) -> Self {
        Self {
            nanoseconds: self.nanoseconds + other.nanoseconds,
            calls: self.calls + other.calls,
        }
    }
}

/// Every capture in the given folders and files, each folder's sorted by name.
fn find_captures(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut captures = Vec::new();

    for path in paths {
        if path.is_dir() {
            captures.extend(captures_in_folder(path)?);
        } else if is_capture(path) {
            captures.push(path.clone());
        } else if path.is_file() {
            bail!("{} is not a .tracy capture", path.display());
        } else {
            bail!("{} does not exist", path.display());
        }
    }

    ensure!(!captures.is_empty(), "no .tracy captures found");
    Ok(captures)
}

fn captures_in_folder(folder: &Path) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(folder).with_context(|| format!("reading {}", folder.display()))?;

    let mut captures = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if is_capture(&path) {
            captures.push(path);
        }
    }

    captures.sort();
    Ok(captures)
}

fn is_capture(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .is_some_and(|extension| extension == CAPTURE_EXTENSION)
}

/// Each capture's file stem, or `folder/stem` when captures from different folders share a
/// stem, or the full path when even that repeats.
fn column_names(captures: &[PathBuf]) -> Vec<String> {
    let mut names = Vec::<String>::new();

    for capture in captures {
        let stem = file_stem(capture);
        let shares_stem = captures
            .iter()
            .filter(|other| file_stem(other) == stem)
            .count()
            > 1;

        let mut name = if shares_stem {
            format!("{}/{stem}", parent_folder_name(capture))
        } else {
            stem
        };
        if names.contains(&name) {
            name = capture.display().to_string();
        }
        names.push(name);
    }
    names
}

fn read_capture_column(capture: &Path, name: String) -> Result<CaptureColumn> {
    let totals_by_zone_name = export_zone_totals(capture)?;
    let frames = totals_by_zone_name
        .get(FRAME_ZONE)
        .map_or(0.0, |totals| totals.calls);

    Ok(CaptureColumn {
        name,
        frames,
        totals_by_zone_name,
    })
}

/// Runs `tracy-csvexport` on the capture and sums each zone name's rows.
fn export_zone_totals(capture: &Path) -> Result<HashMap<String, ZoneTotals>> {
    let output = Command::new(CSV_EXPORT)
        .args(["-s", CSV_SEPARATOR])
        .arg(capture)
        .output()
        .with_context(|| format!("could not start {CSV_EXPORT}"))?;
    ensure!(
        output.status.success(),
        "{CSV_EXPORT} failed on {}: {}",
        capture.display(),
        failure_message(&output)
    );

    let csv = String::from_utf8_lossy(&output.stdout);
    parse_zone_totals(&csv).with_context(|| format!("reading the export of {}", capture.display()))
}

/// What a failed run printed, preferring stderr, or its exit status when it printed nothing.
fn failure_message(output: &Output) -> String {
    let printed = [&output.stderr, &output.stdout]
        .into_iter()
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_owned())
        .find(|text| !text.is_empty());

    printed.unwrap_or_else(|| format!("{} with no output", output.status))
}

/// Parses the export's `name;src_file;src_line;total_ns;...;counts;...` rows.
///
/// Columns are located from the end of each row: the name and source file come first and may
/// hold any text, while the columns after them are always numbers.
fn parse_zone_totals(csv: &str) -> Result<HashMap<String, ZoneTotals>> {
    let mut lines = csv.lines();
    let header = lines
        .next()
        .context("the export is empty")?
        .split(CSV_SEPARATOR)
        .collect::<Vec<_>>();
    let total_column_from_end = column_from_end(&header, "total_ns")?;
    let calls_column_from_end = column_from_end(&header, "counts")?;

    let mut totals_by_zone_name = HashMap::<String, ZoneTotals>::new();
    for line in lines {
        let fields = line.split(CSV_SEPARATOR).collect::<Vec<_>>();
        if fields.len() < header.len() {
            continue;
        }

        let field_from_end = |offset: usize| fields[fields.len() - offset];
        let row = ZoneTotals {
            nanoseconds: parse_number(field_from_end(total_column_from_end))?,
            calls: parse_number(field_from_end(calls_column_from_end))?,
        };

        let totals = totals_by_zone_name.entry(fields[0].to_owned()).or_default();
        *totals = totals.plus(row);
    }
    Ok(totals_by_zone_name)
}

/// How many places from the end of a row `column` sits, counting the last column as 1.
fn column_from_end(header: &[&str], column: &str) -> Result<usize> {
    let index = header
        .iter()
        .position(|name| *name == column)
        .with_context(|| format!("the export has no {column} column"))?;

    Ok(header.len() - index)
}

fn parse_number(text: &str) -> Result<f64> {
    text.parse()
        .with_context(|| format!("`{text}` is not a number"))
}

fn report_table(zones: &[Zone], columns: &[CaptureColumn]) -> Table {
    let capture_names = columns.iter().map(|column| column.name.clone());
    let mut table = Table::new(
        ["Zone".to_owned(), "Unit".to_owned()]
            .into_iter()
            .chain(capture_names),
    );

    let mut frames_row = vec!["frames".to_owned(), "count".to_owned()];
    frames_row.extend(
        columns
            .iter()
            .map(|column| with_thousands_separators(column.frames as u64)),
    );
    table.add_row(frames_row);

    for zone in zones {
        let mut row = vec![zone.label.to_owned(), zone.unit().to_owned()];
        row.extend(
            columns
                .iter()
                .map(|column| milliseconds_cell(zone.milliseconds(column))),
        );
        table.add_row(row);
    }
    table
}

fn milliseconds_cell(milliseconds: Option<f64>) -> String {
    milliseconds.map_or_else(
        || MISSING.to_owned(),
        |milliseconds| format!("{milliseconds:.3}"),
    )
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map_or_else(
        || path.display().to_string(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

fn parent_folder_name(path: &Path) -> String {
    path.parent()
        .and_then(Path::file_name)
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}
