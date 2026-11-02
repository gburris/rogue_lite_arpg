//! The diagnostics overlay F3 cycles: off, compact (the FPS overlay) and detailed (every
//! diagnostic in the store, grouped by path prefix, with a sparkline of its recent history).
//!
//! The overlay only reads [`DiagnosticsStore`], so a new `game/…` metric appears with no change
//! here.

use std::fmt::Write as _;
use std::time::Duration;

use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlaySystems};
use bevy::diagnostic::{
    Diagnostic, DiagnosticsStore, EntityCountDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

const REFRESH_INTERVAL: Duration = Duration::from_millis(100);

/// Most recent measurements drawn in each sparkline.
const SPARKLINE_SAMPLES: usize = 24;

/// Sparkline glyphs from lowest to highest. ASCII, since Bevy's default font has no block glyphs.
const SPARKLINE_LEVELS: &[u8] = b".:-=+*#@";

/// What F3 shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum HudMode {
    Off,
    /// The FPS text and frame-time graph.
    #[default]
    Compact,
    /// Compact plus the diagnostics panel.
    Detailed,
}

impl HudMode {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Compact,
            Self::Compact => Self::Detailed,
            Self::Detailed => Self::Off,
        }
    }
}

/// The panel that lists every diagnostic.
#[derive(Component)]
struct DetailedHud;

/// The text inside [`DetailedHud`].
#[derive(Component)]
struct DetailedHudText;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        EntityCountDiagnosticsPlugin::default(),
        SystemInformationDiagnosticsPlugin,
    ));
    app.add_systems(
        Update,
        (
            // Before the overlay reads its config, so a toggle shows on the same frame.
            cycle_hud_mode.before(FpsOverlaySystems::Customize),
            // The overlay only writes the text of its own `FpsText` entities.
            refresh_detailed_hud
                .run_if(on_timer(REFRESH_INTERVAL))
                .ambiguous_with(FpsOverlaySystems::Customize)
                .ambiguous_with(FpsOverlaySystems::UpdateText),
        ),
    );
}

fn cycle_hud_mode(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut hud_mode: Local<HudMode>,
    mut fps_overlay_config: ResMut<FpsOverlayConfig>,
    detailed_huds: Query<Entity, With<DetailedHud>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F3) {
        return;
    }
    *hud_mode = hud_mode.next();
    fps_overlay_config.enabled = *hud_mode != HudMode::Off;
    for detailed_hud in &detailed_huds {
        commands.entity(detailed_hud).despawn();
    }
    if *hud_mode == HudMode::Detailed {
        commands.spawn((
            DetailedHud,
            Node {
                position_type: PositionType::Absolute,
                right: px(8.0),
                top: px(8.0),
                padding: UiRect::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(i32::MAX - 40),
            Children::spawn_one((
                DetailedHudText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
            )),
        ));
    }
}

fn refresh_detailed_hud(
    diagnostics_store: Res<DiagnosticsStore>,
    mut hud_text: Single<&mut Text, With<DetailedHudText>>,
) {
    let mut diagnostics: Vec<&Diagnostic> = diagnostics_store.iter().collect();
    // Paths without a group (`fps`) come first, then the groups in path order.
    diagnostics.sort_by_key(|diagnostic| {
        let path = diagnostic.path().as_str();
        (path.contains('/'), path.to_owned())
    });

    let mut text = String::new();
    let mut group = "";
    for diagnostic in diagnostics {
        let path = diagnostic.path().as_str();
        let (diagnostic_group, name) = path.split_once('/').unwrap_or(("general", path));
        if diagnostic_group != group {
            group = diagnostic_group;
            writeln!(text, "[{group}]").expect("writing to a String cannot fail");
        }
        let suffix = &diagnostic.suffix;
        writeln!(
            text,
            "{name:<36.36}{:>12.2}{suffix} avg{:>12.2} {}",
            diagnostic.value().unwrap_or(0.0),
            diagnostic.average().unwrap_or(0.0),
            sparkline(diagnostic),
        )
        .expect("writing to a String cannot fail");
    }
    hud_text.0 = text;
}

/// The last [`SPARKLINE_SAMPLES`] measurements scaled between their own minimum and maximum.
fn sparkline(diagnostic: &Diagnostic) -> String {
    let values: Vec<f64> = diagnostic.values().copied().collect();
    let recent = &values[values.len().saturating_sub(SPARKLINE_SAMPLES)..];
    let minimum = recent.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = recent.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = maximum - minimum;
    recent
        .iter()
        .map(|&value| {
            let fraction = if range > 0.0 {
                (value - minimum) / range
            } else {
                0.0
            };
            let level = (fraction * (SPARKLINE_LEVELS.len() - 1) as f64).round() as usize;
            char::from(SPARKLINE_LEVELS[level])
        })
        .collect()
}
