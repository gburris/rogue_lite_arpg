//! Starts the game in the stress arena when it is run with `--stress`.
//!
//! `--stress 1000` swaps the hub for the `StressSwarm` recipe: a small floor with that many
//! enemies, split evenly across kinds and packed around an [`Invincible`] player. The start
//! screen is skipped and vsync is off, so frame time shows real cost.
//!
//! - `--engagement` picks how enemies find the player (see [`Engagement`]).
//! - `--player` picks how the player moves (see [`PlayerScript`]).
//! - `--seconds 60` quits after a warm-up plus that many seconds, and appends one summary line to
//!   `runs/stress_summary.txt` and the log.

use std::collections::BTreeMap;
use std::f32::consts::TAU;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as _;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use clap::{Parser, ValueEnum};

use super::build_info::BuildInfo;
use crate::prelude::*;

/// Recipe the stress arena replaces the hub with.
const RECIPE_NAME: &str = "StressSwarm";

/// Seconds in the arena before measuring starts, which skips the spawn spike and first decisions.
const WARMUP_SECONDS: f32 = 5.0;

/// File the summary line is appended to, since a profiling build has no console.
const SUMMARY_PATH: &str = "runs/stress_summary.txt";

/// Fraction of max speed the scripted player walks at.
const SCRIPTED_SPEED: f32 = 0.5;

/// Seconds for one lap of [`PlayerScript::Circle`].
const CIRCLE_LAP_SECONDS: f32 = 8.0;

/// Seconds for each leg of [`PlayerScript::Strafe`].
const STRAFE_LEG_SECONDS: f32 = 2.0;

/// The arena's enemy count, [`Engagement`] and [`PlayerScript`]. Present only when the game runs
/// with `--stress`.
#[derive(Resource, Clone, Copy, Debug)]
pub struct StressArena {
    enemies_per_kind: u32,
    engagement: Engagement,
    player_script: PlayerScript,
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

/// How the player moves, from `--player`:
///
/// - `still` (default): stands in place.
/// - `circle`: walks a slow circle, so the camera follows and enemies keep turning.
/// - `strafe`: walks left and right.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum PlayerScript {
    #[default]
    Still,
    Circle,
    Strafe,
}

/// The game's command line, such as `--stress 1000 --seconds 30 --engagement sight`. `main`
/// parses it before building the app, so `--help` and bad flags exit before a window opens.
#[derive(Resource, Parser, Clone, Copy, Debug)]
#[command(about = "Baba Yaga. These flags start the stress arena.")]
pub(crate) struct StressArgs {
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

    #[arg(
        long,
        value_enum,
        default_value_t,
        requires = "enemies",
        help = "How the player moves"
    )]
    player: PlayerScript,
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
    let stress_args = app
        .world_mut()
        .remove_resource::<StressArgs>()
        .expect("main inserts StressArgs before adding plugins");
    let Some(requested_enemies) = stress_args.enemies else {
        return;
    };
    let stress_arena = StressArena::new(
        requested_enemies,
        stress_args.engagement,
        stress_args.player,
    );
    info!(
        "Stress arena: {} enemies of each of {} kinds, engagement {}, player {}",
        stress_arena.enemies_per_kind,
        enemy_kind_count(),
        stress_arena.engagement.name(),
        stress_arena.player_script.name()
    );

    app.insert_resource(stress_arena);
    app.add_observer(on_player_added);
    app.add_observer(on_enemy_added);
    app.add_systems(OnEnter(AppState::StartScreen), skip_start_screen);
    app.add_systems(Startup, disable_vsync);
    app.add_systems(
        Update,
        move_scripted_player.in_set(InGameSystems::PlayerInput),
    );

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
    fn new(requested_enemies: u32, engagement: Engagement, player_script: PlayerScript) -> Self {
        Self {
            enemies_per_kind: requested_enemies / enemy_kind_count(),
            engagement,
            player_script,
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

impl PlayerScript {
    /// The name `--player` takes, also written to the summary line.
    fn name(self) -> &'static str {
        match self {
            Self::Still => "still",
            Self::Circle => "circle",
            Self::Strafe => "strafe",
        }
    }

    /// The way the player walks after `seconds` of game time, or `None` to stand still.
    fn direction(self, seconds: f32) -> Option<Vec2> {
        match self {
            Self::Still => None,
            Self::Circle => Some(Vec2::from_angle(seconds / CIRCLE_LAP_SECONDS * TAU)),
            Self::Strafe => {
                let leg = (seconds / STRAFE_LEG_SECONDS) as u32;
                Some(if leg % 2 == 0 { Vec2::X } else { Vec2::NEG_X })
            }
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

fn move_scripted_player(
    stress_arena: Res<StressArena>,
    time: Res<Time>,
    mut player_kinematics: Single<&mut Kinematics, With<Player>>,
) {
    if let Some(direction) = stress_arena.player_script.direction(time.elapsed_secs()) {
        player_kinematics.move_in_direction(direction, SCRIPTED_SPEED);
    }
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

    stress_run.report(*stress_arena);
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
    fn report(&self, stress_arena: StressArena) {
        let summary = self.summary(stress_arena);
        info!("{summary}");
        log_summary_to_file(&summary);
    }

    /// One line: the run's settings, build, frame stats, then the mean of each `game/…` diagnostic.
    fn summary(&self, stress_arena: StressArena) -> String {
        let mut summary = format!(
            "stress summary: arena={} engagement={} player={} seconds={} startup_seconds={:.2}",
            self.total_enemies,
            stress_arena.engagement.name(),
            stress_arena.player_script.name(),
            self.measure_seconds,
            self.started_at.unwrap_or(0.0),
        );

        BuildInfo::read().write_fields(&mut summary);
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
