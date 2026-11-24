use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::remote::list;
use crate::table::Table;

/// Column names of the coverage table, saved and printed.
const COVERAGE_COLUMNS: [&str; 4] = ["kind", "behavior", "share_%", "starts/min"];
/// Starts a behavior would make in a run at its baseline rate before never starting counts as a
/// change. An unchanged behavior expected to start 5 times misses them all about 1 time in 150.
const EXPECTED_STARTS_TO_FLAG: f64 = 5.0;

/// How long each behavior of one enemy kind ran and how often it started, summed over every
/// enemy of that kind in a run's recordings. A behavior counts as started whenever its start time
/// differs from the sample before, including at an enemy's first sample, so one that ends and is
/// chosen again between two samples still counts. One that starts and ends between two samples
/// goes uncounted.
#[derive(Default)]
pub(super) struct BehaviorCoverage {
    alive_seconds: f64,
    behaviors: BTreeMap<String, BehaviorTally>,
}

#[derive(Clone, Copy, Debug, Default)]
struct BehaviorTally {
    seconds: f64,
    starts: u32,
}

impl BehaviorCoverage {
    fn alive_minutes(&self) -> f64 {
        self.alive_seconds / 60.0
    }

    /// Adds one enemy's `(game time, snapshot entry)` samples, in order, up to its death.
    pub(super) fn add_enemy_timeline(&mut self, timeline: &[(f64, &Value)]) {
        let mut previous: Option<(f64, &Value)> = None;
        for &(time, enemy) in timeline {
            if enemy["hp"].as_f64().unwrap_or_default() <= 0.0 {
                break;
            }
            let seconds = previous.map_or(0.0, |(previous_time, _)| time - previous_time);
            self.alive_seconds += seconds;
            for name in behavior_names(enemy) {
                let tally = self.behaviors.entry(name.to_owned()).or_default();
                tally.seconds += seconds;
                let started_at = &enemy["behavior_start_times"][name];
                let previous_started_at =
                    previous.map(|(_, previous)| &previous["behavior_start_times"][name]);
                if previous_started_at != Some(started_at) {
                    tally.starts += 1;
                }
            }
            previous = Some((time, enemy));
        }
    }
}

fn behavior_names(enemy: &Value) -> impl Iterator<Item = &str> {
    list(&enemy["behaviors"]).filter_map(Value::as_str)
}

/// How much of an enemy's life one behavior ran, and how often it started.
#[derive(Clone, Copy, Debug)]
pub(super) struct BehaviorRates {
    share_percent: f64,
    starts_per_minute: f64,
}

/// Each behavior's rates, keyed by `(kind, behavior)`.
pub(super) type CoverageRates = BTreeMap<(String, String), BehaviorRates>;

/// The rates of every behavior in `coverage`, a run's coverage by enemy kind.
pub(super) fn to_coverage_rates(coverage: &BTreeMap<String, BehaviorCoverage>) -> CoverageRates {
    let mut rates = CoverageRates::new();
    for (kind, kind_coverage) in coverage {
        let minutes = kind_coverage.alive_minutes();
        if minutes <= 0.0 {
            continue;
        }
        for (behavior, tally) in &kind_coverage.behaviors {
            rates.insert(
                (kind.clone(), behavior.clone()),
                BehaviorRates {
                    share_percent: 100.0 * tally.seconds / kind_coverage.alive_seconds,
                    starts_per_minute: f64::from(tally.starts) / minutes,
                },
            );
        }
    }
    rates
}

/// The saved coverage table, one row per `(kind, behavior)`.
pub(super) fn to_coverage_tsv(rates: &CoverageRates) -> String {
    let mut text = format!("{}\n", COVERAGE_COLUMNS.join("\t"));
    for ((kind, behavior), rate) in rates {
        let _ = writeln!(
            text,
            "{kind}\t{behavior}\t{:.2}\t{:.2}",
            rate.share_percent, rate.starts_per_minute
        );
    }
    text
}

/// A saved coverage table.
pub(super) fn read_coverage_tsv(path: &Path) -> Result<CoverageRates> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut rates = CoverageRates::new();
    for line in text.lines().skip(1) {
        let cells: Vec<&str> = line.split('\t').collect();
        let [kind, behavior, share_percent, starts_per_minute] = cells.as_slice() else {
            continue;
        };
        let (Ok(share_percent), Ok(starts_per_minute)) =
            (share_percent.parse(), starts_per_minute.parse())
        else {
            continue;
        };
        rates.insert(
            ((*kind).to_owned(), (*behavior).to_owned()),
            BehaviorRates {
                share_percent,
                starts_per_minute,
            },
        );
    }
    Ok(rates)
}

/// The behaviors of the kinds in `coverage` that `is_comparable` accepts that never start in
/// `rates`, though at their `baseline` rate they would have started [`EXPECTED_STARTS_TO_FLAG`]
/// times in this run.
pub(super) fn find_behaviors_no_longer_started(
    coverage: &BTreeMap<String, BehaviorCoverage>,
    is_comparable: impl Fn(&str) -> bool,
    rates: &CoverageRates,
    baseline: &CoverageRates,
) -> Vec<(String, String)> {
    let mut no_longer_started = Vec::new();
    for (key, before) in baseline {
        let Some(kind_coverage) = coverage.get(&key.0) else {
            continue;
        };
        if !is_comparable(&key.0) {
            continue;
        }
        let expected_starts = before.starts_per_minute * kind_coverage.alive_minutes();
        let started_now = rates
            .get(key)
            .is_some_and(|now| now.starts_per_minute > 0.0);
        if expected_starts >= EXPECTED_STARTS_TO_FLAG && !started_now {
            no_longer_started.push(key.clone());
        }
    }
    no_longer_started
}

/// One row per behavior of the kinds in `coverage` that ran in `rates` or in `baseline`, each
/// cell with its change from the baseline.
pub(super) fn coverage_table(
    coverage: &BTreeMap<String, BehaviorCoverage>,
    rates: &CoverageRates,
    baseline: &CoverageRates,
) -> Table {
    let mut table = Table::new(COVERAGE_COLUMNS);
    let baseline_keys = baseline
        .keys()
        .filter(|(kind, _)| coverage.contains_key(kind));
    let mut keys: Vec<&(String, String)> = rates.keys().chain(baseline_keys).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        let now = rates.get(key);
        let before = baseline.get(key);
        let cell = |rate: fn(&BehaviorRates) -> f64| {
            let value = now.map_or(0.0, rate);
            match before {
                Some(before) => format!("{value:.1} ({:+.1})", value - rate(before)),
                None => format!("{value:.1}"),
            }
        };
        table.add_row(vec![
            key.0.clone(),
            key.1.clone(),
            cell(|rate| rate.share_percent),
            cell(|rate| rate.starts_per_minute),
        ]);
    }
    table
}
