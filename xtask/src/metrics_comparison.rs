use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter, Write as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::ai_metrics::{METRICS, ScenarioMetrics};
use crate::behavior_coverage::{CoverageRates, read_coverage_tsv, to_coverage_tsv};
use crate::table::Table;

/// Where in a run's recordings folder each repeat's metrics go.
const REPEATS_FILE_NAME: &str = "repeats.tsv";
/// Where in a run's recordings folder each behavior's share and starts go.
const COVERAGE_FILE_NAME: &str = "behaviors.tsv";
/// Fraction of the baseline repeats' range the noise band adds on each side. With
/// [`BASELINE_REPEATS_FOR_BAND`] baseline repeats and the default repeats of a new run, about 1
/// unchanged median in 150 falls outside it, and a median that moved by half about 1 in 3.
const BAND_WIDENING: f64 = 0.5;
/// Least the band adds on each side, as a fraction of the repeats' midpoint, so a metric whose
/// repeats all agree still allows a small change.
const LEAST_BAND_WIDENING: f64 = 0.1;
/// Repeats of a scenario a baseline needs for the band's chance rate in [`BAND_WIDENING`]; a
/// cell outside a band from fewer is marked but doesn't fail a run.
pub(super) const BASELINE_REPEATS_FOR_BAND: usize = 9;
/// Metrics outside a noise band of [`BASELINE_REPEATS_FOR_BAND`] repeats that fail a run. An
/// unchanged run of every kind and script has about one by chance.
pub(super) const CELLS_OUTSIDE_BAND_TO_FAIL: usize = 3;
/// Decimal places the saved tables keep.
const SAVED_DECIMALS: usize = 2;

/// A saved table, keyed by `(kind, script)`, each row's values by metric name.
type SavedMetrics = BTreeMap<(String, String), BTreeMap<String, f64>>;
/// Each scenario's noise bands, keyed by `(kind, script)`, then by metric name.
type NoiseBands = BTreeMap<(String, String), BTreeMap<String, NoiseBand>>;

/// A saved run to compare with: its medians, the noise band of each, and its behavior coverage.
#[derive(Default)]
pub(super) struct Baseline {
    metrics: SavedMetrics,
    bands: NoiseBands,
    pub(super) coverage: CoverageRates,
}

impl Baseline {
    /// Whether the baseline played the same scripts against `kind` as `scenarios` do, so its
    /// coverage rates, which average over every script, compare.
    pub(super) fn played_same_scripts(&self, scenarios: &[ScenarioMetrics], kind: &str) -> bool {
        let scripts_now: BTreeSet<&str> = scenarios
            .iter()
            .filter(|scenario| scenario.kind == kind)
            .map(|scenario| scenario.script.name())
            .collect();
        let scripts_before: BTreeSet<&str> = self
            .metrics
            .keys()
            .filter(|(baseline_kind, _)| baseline_kind == kind)
            .map(|(_, script)| script.as_str())
            .collect();
        scripts_now == scripts_before
    }

    /// How many of `scenarios` have a metric with no noise band of
    /// [`BASELINE_REPEATS_FOR_BAND`] repeats, so its cells can't fail the run.
    pub(super) fn count_scenarios_without_full_band(&self, scenarios: &[ScenarioMetrics]) -> usize {
        scenarios
            .iter()
            .filter(|scenario| {
                METRICS.iter().any(|metric| {
                    self.find_noise_band(scenario, metric)
                        .is_none_or(|band| !band.has_full_repeats())
                })
            })
            .count()
    }

    fn find_noise_band(&self, scenario: &ScenarioMetrics, metric: &str) -> Option<NoiseBand> {
        self.bands
            .get(&scenario.key())
            .and_then(|scenario_bands| scenario_bands.get(metric))
            .copied()
    }
}

/// The range a scenario's median falls in by chance: the baseline's repeats from lowest to
/// highest, widened by [`BAND_WIDENING`] and at least [`LEAST_BAND_WIDENING`].
#[derive(Clone, Copy, Debug)]
struct NoiseBand {
    low: f64,
    high: f64,
    repeats: usize,
}

impl NoiseBand {
    fn from_repeats(values: &[f64]) -> Option<Self> {
        let lowest = values.iter().copied().min_by(f64::total_cmp)?;
        let highest = values.iter().copied().max_by(f64::total_cmp)?;
        let widening = (BAND_WIDENING * (highest - lowest))
            .max(LEAST_BAND_WIDENING * f64::midpoint(lowest, highest).abs());
        Some(Self {
            low: lowest - widening,
            high: highest + widening,
            repeats: values.len(),
        })
    }

    fn has_full_repeats(self) -> bool {
        self.repeats >= BASELINE_REPEATS_FOR_BAND
    }

    fn contains(self, value: f64) -> bool {
        // The band's repeats were saved rounded, so a value may differ by half the last digit.
        let rounding = 0.5 * 10f64.powi(-(SAVED_DECIMALS as i32));
        (self.low - rounding..=self.high + rounding).contains(&value)
    }
}

/// A scenario's median that fell outside the baseline's noise band.
pub(super) struct CellOutsideBand {
    kind: String,
    script: &'static str,
    metric: &'static str,
    value: f64,
    band: NoiseBand,
}

impl CellOutsideBand {
    /// Whether its band has [`BASELINE_REPEATS_FOR_BAND`] repeats, so it counts toward failing.
    pub(super) fn counts_toward_failure(&self) -> bool {
        self.band.has_full_repeats()
    }
}

impl Display for CellOutsideBand {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} {} {} is {:.2}, outside {:.2} to {:.2}",
            self.kind, self.script, self.metric, self.value, self.band.low, self.band.high
        )
    }
}

/// Saves `recordings`' tables: the medians as `<recordings>.tsv` beside the folder, and every
/// repeat's values and each behavior's rates inside it, where the baseline search doesn't look.
/// Returns the medians' file.
pub(super) fn save_tables(
    recordings: &Path,
    scenarios: &[ScenarioMetrics],
    coverage_rates: &CoverageRates,
) -> Result<PathBuf> {
    let metrics_file = recordings.with_added_extension("tsv");
    let files = [
        (metrics_file.clone(), to_metrics_tsv(scenarios)),
        (
            recordings.join(REPEATS_FILE_NAME),
            to_repeats_tsv(scenarios),
        ),
        (
            recordings.join(COVERAGE_FILE_NAME),
            to_coverage_tsv(coverage_rates),
        ),
    ];
    for (file, text) in files {
        fs::write(&file, text).with_context(|| format!("writing {}", file.display()))?;
    }
    Ok(metrics_file)
}

/// The baseline saved as `metrics_file`, with the noise bands and coverage in the folder of its
/// name.
pub(super) fn read_baseline(metrics_file: &Path) -> Result<Baseline> {
    let folder = metrics_file.with_extension("");
    Ok(Baseline {
        metrics: read_metrics_tsv(metrics_file)?,
        bands: read_noise_bands(&folder.join(REPEATS_FILE_NAME))?,
        coverage: read_coverage_tsv(&folder.join(COVERAGE_FILE_NAME))?,
    })
}

/// The newest `.tsv` beside `current` other than `current` itself.
pub(super) fn newest_other_metrics_file(current: &Path) -> Result<Option<PathBuf>> {
    let directory = current
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "tsv") || path == current {
            continue;
        }
        let modified = path.metadata()?.modified()?;
        if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
            newest = Some((modified, path));
        }
    }
    Ok(newest.map(|(_, path)| path))
}

fn to_metrics_tsv(scenarios: &[ScenarioMetrics]) -> String {
    let mut text = format!("kind\tscript\t{}\n", METRICS.join("\t"));
    for scenario in scenarios {
        let _ = writeln!(
            text,
            "{}\t{}\t{}",
            scenario.kind,
            scenario.script.name(),
            to_tsv_cells(&scenario.medians())
        );
    }
    text
}

fn to_repeats_tsv(scenarios: &[ScenarioMetrics]) -> String {
    let mut text = format!("kind\tscript\trepeat\t{}\n", METRICS.join("\t"));
    for scenario in scenarios {
        for (index, values) in scenario.repeats.iter().enumerate() {
            let _ = writeln!(
                text,
                "{}\t{}\t{}\t{}",
                scenario.kind,
                scenario.script.name(),
                index + 1,
                to_tsv_cells(values)
            );
        }
    }
    text
}

fn to_tsv_cells(values: &[f64]) -> String {
    let cells: Vec<String> = values
        .iter()
        .map(|value| format!("{value:.SAVED_DECIMALS$}"))
        .collect();
    cells.join("\t")
}

fn read_metrics_tsv(path: &Path) -> Result<SavedMetrics> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let mut saved = SavedMetrics::new();
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let [kind, script, values @ ..] = cells.as_slice() else {
            continue;
        };
        let values = header
            .iter()
            .skip(2)
            .zip(values)
            .filter_map(|(name, value)| Some(((*name).to_owned(), value.parse().ok()?)))
            .collect();
        saved.insert(((*kind).to_owned(), (*script).to_owned()), values);
    }
    Ok(saved)
}

/// The noise bands of a saved `repeats.tsv`.
fn read_noise_bands(repeats_file: &Path) -> Result<NoiseBands> {
    let text = fs::read_to_string(repeats_file)
        .with_context(|| format!("reading {}", repeats_file.display()))?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let mut repeats: BTreeMap<(String, String), BTreeMap<String, Vec<f64>>> = BTreeMap::new();
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let [kind, script, _repeat, values @ ..] = cells.as_slice() else {
            continue;
        };
        let scenario_repeats = repeats
            .entry(((*kind).to_owned(), (*script).to_owned()))
            .or_default();
        for (name, value) in header.iter().skip(3).zip(values) {
            if let Ok(value) = value.parse() {
                scenario_repeats
                    .entry((*name).to_owned())
                    .or_default()
                    .push(value);
            }
        }
    }
    let mut bands = NoiseBands::new();
    for (scenario, metrics) in repeats {
        let scenario_bands = metrics
            .into_iter()
            .filter_map(|(name, values)| Some((name, NoiseBand::from_repeats(&values)?)))
            .collect();
        bands.insert(scenario, scenario_bands);
    }
    Ok(bands)
}

/// Each scenario median outside its noise band in `baseline`.
pub(super) fn find_cells_outside_band(
    scenarios: &[ScenarioMetrics],
    baseline: &Baseline,
) -> Vec<CellOutsideBand> {
    let mut cells = Vec::new();
    for scenario in scenarios {
        for (metric, value) in METRICS.iter().zip(scenario.medians()) {
            let Some(band) = baseline.find_noise_band(scenario, metric) else {
                continue;
            };
            if !band.contains(value) {
                cells.push(CellOutsideBand {
                    kind: scenario.kind.clone(),
                    script: scenario.script.name(),
                    metric,
                    value,
                    band,
                });
            }
        }
    }
    cells
}

/// One row per scenario; each cell is the new median with its change from the baseline, and
/// `!` when it falls outside the noise band.
pub(super) fn comparison_table(scenarios: &[ScenarioMetrics], baseline: &Baseline) -> Table {
    let mut table = Table::new(["kind", "script"].into_iter().chain(METRICS));
    for scenario in scenarios {
        let before = baseline.metrics.get(&scenario.key());
        let mut cells = vec![scenario.kind.clone(), scenario.script.name().to_owned()];
        for (metric, value) in METRICS.iter().zip(scenario.medians()) {
            let mut cell = match before.and_then(|before| before.get(*metric)) {
                Some(before) => format!("{value:.1} ({:+.1})", value - before),
                None => format!("{value:.1}"),
            };
            let band = baseline.find_noise_band(scenario, metric);
            if band.is_some_and(|band| !band.contains(value)) {
                cell.push_str(" !");
            }
            cells.push(cell);
        }
        table.add_row(cells);
    }
    table
}
