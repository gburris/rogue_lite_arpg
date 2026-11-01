//! Starts the game in the stress arena when `STRESS_ARENA` is set.
//!
//! `STRESS_ARENA=warrior=250,ice_mage=250,fire_mage=250,bat=250` replaces the hub request with
//! the `StressArena` recipe, spawns that many of each listed kind (unlisted kinds spawn none),
//! and makes the player [`Invincible`]. Dev builds step the counts with F2.
//!
//! `STRESS_SECONDS=60` also quits after a warm-up plus that many seconds in the arena, and
//! appends one summary line to `runs/stress_summary.txt` and the log.

use std::fs::{self, OpenOptions};
use std::io::Write as _;

use bevy::diagnostic::{Diagnostic, DiagnosticPath, DiagnosticsStore};
use bevy::prelude::*;

use super::build::MapRequest;
use super::placement::CountRange;
use super::recipe::{MapRecipe, SpawnKind};
use crate::prelude::*;

/// Name of the recipe `STRESS_ARENA` loads in place of the hub.
pub(super) const STRESS_ARENA_RECIPE: &str = "StressArena";

/// Total enemy counts F2 steps through, split evenly across the enemy kinds.
#[cfg(feature = "dev")]
const STEP_TOTALS: [u32; 4] = [100, 200, 500, 1000];

/// Seconds in the arena before measuring starts, which skips the spawn spike and first decisions.
const WARMUP_SECONDS: f32 = 5.0;

/// File the summary line is appended to, since a profiling build has no console.
const SUMMARY_PATH: &str = "runs/stress_summary.txt";

/// How many enemies of one kind the stress arena spawns.
#[derive(Clone, Copy, Debug)]
struct KindCount {
    kind: EnemyKind,
    count: u32,
}

/// Enemy counts parsed from `STRESS_ARENA`. Present only when the variable is set.
#[derive(Resource, Debug)]
pub(super) struct StressArena {
    kind_counts: Vec<KindCount>,
    #[cfg(feature = "dev")]
    next_step: usize,
}

/// Frame and decision costs collected while `STRESS_SECONDS` is set.
#[derive(Resource)]
struct StressRun {
    arena_spec: String,
    measure_seconds: f32,
    started_at: Option<f32>,
    frame_ms: Vec<f32>,
    decide_ms: Vec<f32>,
    brains: Vec<f32>,
}

pub(super) fn plugin(app: &mut App) {
    let seconds = std::env::var("STRESS_SECONDS").ok();
    let Ok(arena_spec) = std::env::var("STRESS_ARENA") else {
        if seconds.is_some() {
            warn!("STRESS_SECONDS is ignored without STRESS_ARENA");
        }
        return;
    };
    let kind_counts = parse_kind_counts(&arena_spec)
        .unwrap_or_else(|error| panic!("STRESS_ARENA=`{arena_spec}` is invalid: {error}"));
    info!("Stress arena enemy counts: {kind_counts:?}");
    app.insert_resource(StressArena {
        kind_counts,
        #[cfg(feature = "dev")]
        next_step: 0,
    });
    app.add_observer(on_player_added);

    if let Some(seconds) = seconds {
        let measure_seconds = seconds
            .trim()
            .parse::<f32>()
            .unwrap_or_else(|error| panic!("STRESS_SECONDS=`{seconds}` is invalid: {error}"));
        app.insert_resource(StressRun {
            arena_spec,
            measure_seconds,
            started_at: None,
            frame_ms: Vec::new(),
            decide_ms: Vec::new(),
            brains: Vec::new(),
        });
        app.add_systems(
            Last,
            sample_stress_run
                .run_if(resource_exists::<StressRun>.and_then(in_state(AppState::Playing))),
        );
    }
}

/// Sends the game's first map request, the hub, to the stress arena.
pub(super) fn redirect_hub(request: MapRequest) -> MapRequest {
    match request {
        MapRequest::Hub => MapRequest::Named {
            recipe: STRESS_ARENA_RECIPE.to_owned(),
            seed: None,
        },
        MapRequest::Weighted | MapRequest::Named { .. } => request,
    }
}

impl StressArena {
    /// Returns the stress arena recipe with its enemy counts replaced, or `None` for any
    /// other recipe.
    pub(super) fn override_counts(&self, recipe: &MapRecipe) -> Option<MapRecipe> {
        if recipe.name != STRESS_ARENA_RECIPE {
            return None;
        }
        let mut recipe = recipe.clone();
        recipe.spawns.retain_mut(|spawn| {
            let SpawnKind::Enemy(kind) = spawn.what else {
                return true;
            };
            let count = self
                .kind_counts
                .iter()
                .rev()
                .find(|kind_count| kind_count.kind == kind)
                .map_or(0, |kind_count| kind_count.count);
            spawn.count = CountRange::exactly(count);
            count > 0
        });
        Some(recipe)
    }

    /// Moves to the next total in [`STEP_TOTALS`], wrapping, split evenly across every kind.
    #[cfg(feature = "dev")]
    pub(super) fn advance_step(&mut self) {
        let total = STEP_TOTALS[self.next_step % STEP_TOTALS.len()];
        self.next_step += 1;
        let kinds = u32::try_from(EnemyKind::ALL.len()).expect("enemy kind count fits in u32");
        self.kind_counts = EnemyKind::ALL
            .iter()
            .map(|&kind| KindCount {
                kind,
                count: total / kinds,
            })
            .collect();
        info!("Stress arena stepped to {total} enemies");
    }
}

fn on_player_added(player_added: On<Add, Player>, mut commands: Commands) {
    commands.entity(player_added.entity).insert(Invincible);
}

/// Records each frame once the warm-up is over, then logs the summary and quits.
fn sample_stress_run(
    mut commands: Commands,
    mut stress_run: ResMut<StressRun>,
    real_time: Res<Time<Real>>,
    diagnostics: Res<DiagnosticsStore>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let now = real_time.elapsed_secs();
    let started_at = *stress_run.started_at.get_or_insert(now);
    let elapsed = now - started_at;
    if elapsed < WARMUP_SECONDS {
        return;
    }

    let latest = |path: &DiagnosticPath| {
        diagnostics
            .get(path)
            .and_then(Diagnostic::value)
            .unwrap_or(0.0) as f32
    };
    stress_run.frame_ms.push(real_time.delta_secs() * 1000.0);
    stress_run.decide_ms.push(latest(&DECIDE_MS));
    stress_run.brains.push(latest(&BRAINS));

    if elapsed < WARMUP_SECONDS + stress_run.measure_seconds {
        return;
    }
    let summary = stress_run.summary();
    info!("{summary}");
    log_summary_to_file(&summary);
    commands.remove_resource::<StressRun>();
    app_exit.write(AppExit::Success);
}

impl StressRun {
    /// One line: the arena spec, frames measured, frame time median and p99, and mean decide
    /// time and brain count.
    fn summary(&self) -> String {
        let mut sorted_frame_ms = self.frame_ms.clone();
        sorted_frame_ms.sort_by(f32::total_cmp);
        let frames = sorted_frame_ms.len();
        let median = sorted_frame_ms[frames / 2];
        let p99 = sorted_frame_ms[(frames * 99).div_ceil(100) - 1];
        format!(
            "stress summary: arena={} seconds={} frames={frames} median_frame_ms={median:.2} \
             p99_frame_ms={p99:.2} mean_decide_ms={:.3} mean_brains={:.0}",
            self.arena_spec,
            self.measure_seconds,
            mean(&self.decide_ms),
            mean(&self.brains),
        )
    }
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

fn log_summary_to_file(summary: &str) {
    let result = fs::create_dir_all("runs").and_then(|()| {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(SUMMARY_PATH)?;
        writeln!(file, "{summary}")
    });
    if let Err(error) = result {
        error!("Could not write {SUMMARY_PATH}: {error}");
    }
}

/// Parses `kind=count,kind=count`. Kind names ignore case and underscores (`ice_mage`).
fn parse_kind_counts(value: &str) -> Result<Vec<KindCount>, String> {
    let mut kind_counts = Vec::new();
    for entry in value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let (name, count) = entry
            .split_once('=')
            .ok_or_else(|| format!("`{entry}` is not kind=count"))?;
        let kind = enemy_kind_named(name.trim())?;
        let count = count
            .trim()
            .parse::<u32>()
            .map_err(|error| format!("count in `{entry}`: {error}"))?;
        kind_counts.push(KindCount { kind, count });
    }
    Ok(kind_counts)
}

fn enemy_kind_named(name: &str) -> Result<EnemyKind, String> {
    let normalize = |text: &str| text.replace('_', "").to_lowercase();
    EnemyKind::ALL
        .iter()
        .copied()
        .find(|kind| normalize(&format!("{kind:?}")) == normalize(name))
        .ok_or_else(|| {
            let valid: Vec<String> = EnemyKind::ALL
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect();
            format!("unknown enemy kind `{name}`, expected one of {valid:?}")
        })
}
