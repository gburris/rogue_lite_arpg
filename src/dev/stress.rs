//! Starts the game in the stress arena when `STRESS_ARENA` is set.
//!
//! `STRESS_ARENA=1000` replaces the hub request with the `StressSwarm` recipe, a small floor
//! with the enemies packed around the player, spawns that many enemies split evenly across
//! every kind, and makes the player
//! [`Invincible`]. The start screen is skipped and vsync is off, so frame time shows real cost.
//!
//! `STRESS_SECONDS=60` also quits after a warm-up plus that many seconds in the arena, and
//! appends one summary line to `runs/stress_summary.txt` and the log.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as _;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};

use super::diagnostics::FIXED_STEPS;
use crate::prelude::*;

/// Recipe the stress arena replaces the hub with.
const RECIPE_NAME: &str = "StressSwarm";

/// Seconds in the arena before measuring starts, which skips the spawn spike and first decisions.
const WARMUP_SECONDS: f32 = 5.0;

/// File the summary line is appended to, since a profiling build has no console.
const SUMMARY_PATH: &str = "runs/stress_summary.txt";

/// Enemy count parsed from `STRESS_ARENA`. Present only when the variable is set.
#[derive(Resource, Debug)]
pub struct StressArena {
    enemies_per_kind: u32,
}

/// Frame times and `game/…` diagnostics collected while `STRESS_SECONDS` is set.
#[derive(Resource)]
struct StressRun {
    arena_spec: String,
    measure_seconds: f32,
    started_at: Option<f32>,
    frame_ms: Vec<f32>,
    frame_ms_with_fixed_step: Vec<f32>,
    frame_ms_without_fixed_step: Vec<f32>,
    diagnostic_values: BTreeMap<String, Vec<f32>>,
}

pub(super) fn plugin(app: &mut App) {
    let seconds = std::env::var("STRESS_SECONDS").ok();
    let Ok(arena_spec) = std::env::var("STRESS_ARENA") else {
        if seconds.is_some() {
            warn!("STRESS_SECONDS is ignored without STRESS_ARENA");
        }
        return;
    };
    let total_enemies = arena_spec
        .trim()
        .parse::<u32>()
        .unwrap_or_else(|error| panic!("STRESS_ARENA=`{arena_spec}` is invalid: {error}"));
    let kinds = u32::try_from(EnemyKind::ALL.len()).expect("enemy kind count fits in u32");
    let enemies_per_kind = total_enemies / kinds;
    info!("Stress arena: {enemies_per_kind} enemies of each of {kinds} kinds");
    app.insert_resource(StressArena { enemies_per_kind });
    app.add_observer(on_player_added);
    app.add_systems(OnEnter(AppState::StartScreen), skip_start_screen);
    app.add_systems(Startup, disable_vsync);

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
            frame_ms_with_fixed_step: Vec::new(),
            frame_ms_without_fixed_step: Vec::new(),
            diagnostic_values: BTreeMap::new(),
        });
        app.add_systems(
            Last,
            sample_stress_run
                .run_if(resource_exists::<StressRun>.and_then(in_state(AppState::Playing))),
        );
    }
}

impl StressArena {
    /// Name of the recipe this run uses in place of the hub.
    pub fn recipe_name(&self) -> &'static str {
        RECIPE_NAME
    }

    /// Sends the game's first map request, the hub, to the stress recipe.
    pub fn redirect_hub(&self, request: MapRequest) -> MapRequest {
        match request {
            MapRequest::Hub => MapRequest::Named {
                recipe: self.recipe_name().to_owned(),
                seed: None,
            },
            MapRequest::Weighted | MapRequest::Named { .. } => request,
        }
    }

    /// Returns the stress recipe with its enemy counts replaced, or `None` for any other recipe.
    pub fn override_counts(&self, recipe: &MapRecipe) -> Option<MapRecipe> {
        if recipe.name() != self.recipe_name() {
            return None;
        }
        Some(
            recipe
                .clone()
                .with_enemy_counts(|_kind| self.enemies_per_kind),
        )
    }
}

/// Starts the game as if the start button was pressed, so a sweep needs no input.
fn skip_start_screen(mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::AssetLoading);
}

/// Uncaps the frame rate, so frame time reflects cost instead of the display refresh rate.
fn disable_vsync(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.present_mode = PresentMode::AutoNoVsync;
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

    let frame_ms = real_time.delta_secs() * 1000.0;
    let mut had_fixed_step = false;
    for diagnostic in diagnostics.iter() {
        let path = diagnostic.path();
        if !path.as_str().starts_with("game/") {
            continue;
        }
        let value = diagnostic.value().unwrap_or(0.0) as f32;
        if *path == FIXED_STEPS {
            had_fixed_step = value > 0.0;
        }
        stress_run
            .diagnostic_values
            .entry(path.to_string())
            .or_default()
            .push(value);
    }
    stress_run.frame_ms.push(frame_ms);
    if had_fixed_step {
        stress_run.frame_ms_with_fixed_step.push(frame_ms);
    } else {
        stress_run.frame_ms_without_fixed_step.push(frame_ms);
    }

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
    /// One line: the arena spec, frames measured, median and p99 frame time overall and split by
    /// whether the frame ran a fixed step, then the mean of every `game/…` diagnostic.
    fn summary(&self) -> String {
        let mut summary = format!(
            "stress summary: arena={} seconds={}",
            self.arena_spec, self.measure_seconds,
        );
        for (label, frame_ms) in [
            ("", &self.frame_ms),
            ("fixed_", &self.frame_ms_with_fixed_step),
            ("no_fixed_", &self.frame_ms_without_fixed_step),
        ] {
            let mut sorted_frame_ms = frame_ms.clone();
            sorted_frame_ms.sort_by(f32::total_cmp);
            let median = percentile(&sorted_frame_ms, 50);
            let p99 = percentile(&sorted_frame_ms, 99);
            write!(
                summary,
                " {label}frames={} {label}median_frame_ms={median} {label}p99_frame_ms={p99}",
                sorted_frame_ms.len(),
            )
            .expect("writing to a String cannot fail");
        }
        for (path, values) in &self.diagnostic_values {
            write!(summary, " mean_{path}={:.3}", mean(values))
                .expect("writing to a String cannot fail");
        }
        summary
    }
}

/// The given percentile of sorted values with two decimals, or `-` when there are none.
fn percentile(sorted_values: &[f32], percent: usize) -> String {
    let count = sorted_values.len();
    if count == 0 {
        return "-".to_owned();
    }
    let index = (count * percent).div_ceil(100).max(1) - 1;
    format!("{:.2}", sorted_values[index])
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
