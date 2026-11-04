//! Starts the game in the stress arena when it is run with `--stress`.
//!
//! `--stress 1000` swaps the hub for the `StressSwarm` recipe: a small floor with that many
//! enemies, split evenly across kinds and packed around an [`Invincible`] player. The start
//! screen is skipped and vsync is off, so frame time shows real cost.
//!
//! - `--engagement` picks how enemies find the player (see [`Engagement`]).
//! - `--seconds 60` quits after a warm-up plus that many seconds, and appends one summary line to
//!   `runs/stress_summary.txt` and the log.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as _;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use clap::{Parser, ValueEnum};

use crate::prelude::*;

/// Recipe the stress arena replaces the hub with.
const RECIPE_NAME: &str = "StressSwarm";

/// Seconds in the arena before measuring starts, which skips the spawn spike and first decisions.
const WARMUP_SECONDS: f32 = 5.0;

/// File the summary line is appended to, since a profiling build has no console.
const SUMMARY_PATH: &str = "runs/stress_summary.txt";

/// The arena's enemy count and [`Engagement`]. Present only when the game runs with `--stress`.
#[derive(Resource, Clone, Copy, Debug)]
pub struct StressArena {
    enemies_per_kind: u32,
    engagement: Engagement,
}

/// How enemies find the player, from `--engagement`:
///
/// - `all` (default): every enemy targets the player from spawn and keeps the target, since the
///   idle player never hits back. Every run has the same fight.
/// - `sight`: enemies engage on sight, as in normal play. Runs get a heavy or a calm fight by
///   chance.
/// - `none`: enemies are blind and only wander, to measure the crowd without combat.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum Engagement {
    #[default]
    All,
    Sight,
    #[value(name = "none")]
    Blind,
}

/// The game's command line, such as `--stress 1000 --seconds 30 --engagement sight`.
#[derive(Parser, Debug)]
#[command(about = "Baba Yaga. These flags start the stress arena.")]
struct StressArgs {
    #[arg(
        long = "stress",
        value_name = "ENEMIES",
        help = "Start in the stress arena with this many enemies"
    )]
    enemies: Option<u32>,

    #[arg(
        long,
        requires = "enemies",
        help = "Quit after the 5 s warm-up plus this many seconds, writing a summary line"
    )]
    seconds: Option<f32>,

    #[arg(
        long,
        value_enum,
        default_value_t,
        requires = "enemies",
        help = "How enemies find the player"
    )]
    engagement: Engagement,
}

/// Frame times and `game/…` diagnostics collected while `--seconds` is set.
#[derive(Resource)]
struct StressRun {
    total_enemies: u32,
    measure_seconds: f32,
    started_at: Option<f32>,
    frame_ms: Vec<f32>,
    diagnostic_values: BTreeMap<String, Vec<f32>>,
}

/// Order statistics of one set of frame times.
#[derive(Clone, Copy, Debug)]
struct FrameStats {
    frames: usize,
    median_ms: Option<f32>,
    p99_ms: Option<f32>,
    max_ms: Option<f32>,
}

pub(super) fn plugin(app: &mut App) {
    let stress_args = StressArgs::parse();
    let Some(requested_enemies) = stress_args.enemies else {
        return;
    };
    let stress_arena = StressArena::new(requested_enemies, stress_args.engagement);
    info!(
        "Stress arena: {} enemies of each of {} kinds, engagement {}",
        stress_arena.enemies_per_kind,
        enemy_kind_count(),
        stress_arena.engagement.name()
    );

    app.insert_resource(stress_arena);
    app.add_observer(on_player_added);
    app.add_observer(on_enemy_added);
    app.add_systems(OnEnter(AppState::StartScreen), skip_start_screen);
    app.add_systems(Startup, disable_vsync);

    let Some(measure_seconds) = stress_args.seconds else {
        return;
    };
    app.insert_resource(StressRun::new(
        stress_arena.total_enemies(),
        measure_seconds,
    ));
    app.add_systems(
        Last,
        sample_stress_run
            .run_if(resource_exists::<StressRun>.and_then(in_state(AppState::Playing))),
    );
}

fn enemy_kind_count() -> u32 {
    u32::try_from(EnemyKind::ALL.len()).expect("enemy kind count fits in u32")
}

impl StressArena {
    /// Splits the requested enemies evenly across kinds, dropping the remainder.
    fn new(requested_enemies: u32, engagement: Engagement) -> Self {
        Self {
            enemies_per_kind: requested_enemies / enemy_kind_count(),
            engagement,
        }
    }

    fn total_enemies(self) -> u32 {
        self.enemies_per_kind * enemy_kind_count()
    }

    /// Sends the game's first map request, the hub, to the stress recipe.
    pub(crate) fn redirect_hub(request: MapRequest) -> MapRequest {
        match request {
            MapRequest::Hub => MapRequest::Named {
                recipe: RECIPE_NAME.to_owned(),
                seed: None,
            },
            MapRequest::Weighted | MapRequest::Named { .. } => request,
        }
    }

    /// Returns the stress recipe with its enemy counts replaced, or `None` for any other recipe.
    pub(crate) fn override_counts(self, recipe: &MapRecipe) -> Option<MapRecipe> {
        if recipe.name() != RECIPE_NAME {
            return None;
        }
        Some(
            recipe
                .clone()
                .with_enemy_counts(|_kind| self.enemies_per_kind),
        )
    }
}

impl Engagement {
    /// The name `--engagement` takes, also written to the summary line.
    fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Sight => "sight",
            Self::Blind => "none",
        }
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

/// Applies the run's [`Engagement`] to each enemy as it spawns.
fn on_enemy_added(
    enemy_added: On<Add, Enemy>,
    mut commands: Commands,
    stress_arena: Res<StressArena>,
    player: Single<Entity, With<Player>>,
) {
    let mut enemy = commands.entity(enemy_added.entity);
    match stress_arena.engagement {
        Engagement::All => {
            enemy.insert((Targeting(*player), TargetLock));
        }
        Engagement::Sight => {}
        // A zero-width cone never contains the player, but the sight rays still run.
        Engagement::Blind => {
            enemy.insert(VisionCone { half_angle: 0.0 });
        }
    }
}

/// Records each frame once the warm-up is over, then reports the run and quits.
fn sample_stress_run(
    mut commands: Commands,
    mut stress_run: ResMut<StressRun>,
    stress_arena: Res<StressArena>,
    real_time: Res<Time<Real>>,
    diagnostics: Res<DiagnosticsStore>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let now = real_time.elapsed_secs();
    let elapsed = now - *stress_run.started_at.get_or_insert(now);
    if elapsed < WARMUP_SECONDS {
        return;
    }

    stress_run.frame_ms.push(real_time.delta_secs() * 1000.0);
    stress_run.record_diagnostics(&diagnostics);
    if elapsed < WARMUP_SECONDS + stress_run.measure_seconds {
        return;
    }

    stress_run.report(stress_arena.engagement);
    commands.remove_resource::<StressRun>();
    app_exit.write(AppExit::Success);
}

impl StressRun {
    fn new(total_enemies: u32, measure_seconds: f32) -> Self {
        Self {
            total_enemies,
            measure_seconds,
            started_at: None,
            frame_ms: Vec::new(),
            diagnostic_values: BTreeMap::new(),
        }
    }

    /// Stores this frame's value of every `game/…` diagnostic, with 0 for a missing value.
    fn record_diagnostics(&mut self, diagnostics: &DiagnosticsStore) {
        for diagnostic in diagnostics.iter() {
            let path = diagnostic.path();
            if !path.as_str().starts_with("game/") {
                continue;
            }
            self.diagnostic_values
                .entry(path.to_string())
                .or_default()
                .push(diagnostic.value().unwrap_or(0.0) as f32);
        }
    }

    /// Logs the summary and appends it to [`SUMMARY_PATH`].
    fn report(&self, engagement: Engagement) {
        let summary = self.summary(engagement);
        info!("{summary}");
        log_summary_to_file(&summary);
    }

    /// One line: the run's settings, frame stats, then the mean of every `game/…` diagnostic.
    fn summary(&self, engagement: Engagement) -> String {
        let mut summary = format!(
            "stress summary: arena={} engagement={} seconds={} startup_seconds={:.2}",
            self.total_enemies,
            engagement.name(),
            self.measure_seconds,
            self.started_at.unwrap_or(0.0),
        );

        FrameStats::from_frame_times(&self.frame_ms).write_fields(&mut summary);
        for (path, values) in &self.diagnostic_values {
            write!(summary, " mean_{path}={:.3}", mean(values))
                .expect("writing to a String cannot fail");
        }
        summary
    }
}

impl FrameStats {
    fn from_frame_times(frame_ms: &[f32]) -> Self {
        let mut sorted_frame_ms = frame_ms.to_vec();
        sorted_frame_ms.sort_by(f32::total_cmp);
        Self {
            frames: sorted_frame_ms.len(),
            median_ms: percentile(&sorted_frame_ms, 50),
            p99_ms: percentile(&sorted_frame_ms, 99),
            max_ms: sorted_frame_ms.last().copied(),
        }
    }

    /// Appends the frame count, median, p99 and slowest frame.
    fn write_fields(&self, summary: &mut String) {
        write!(
            summary,
            " frames={} median_frame_ms={} p99_frame_ms={} max_frame_ms={}",
            self.frames,
            format_ms(self.median_ms),
            format_ms(self.p99_ms),
            format_ms(self.max_ms),
        )
        .expect("writing to a String cannot fail");
    }
}

/// The given percentile of sorted values, or `None` when there are none.
fn percentile(sorted_values: &[f32], percent: usize) -> Option<f32> {
    let count = sorted_values.len();
    if count == 0 {
        return None;
    }
    let index = (count * percent).div_ceil(100).max(1) - 1;
    Some(sorted_values[index])
}

/// Milliseconds with two decimals, or `-` when there was no value.
fn format_ms(milliseconds: Option<f32>) -> String {
    milliseconds.map_or_else(
        || "-".to_owned(),
        |milliseconds| format!("{milliseconds:.2}"),
    )
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
