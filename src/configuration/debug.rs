use std::fmt::Write as _;

use avian2d::prelude::*;
use bevy::color::palettes::css::{GREEN, RED, YELLOW};
#[cfg(not(target_arch = "wasm32"))]
use bevy::dev_tools::fps_overlay::{FpsOverlayPlugin, FpsOverlaySystems};
use bevy::dev_tools::states::log_transitions;
use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::view;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::INFO,
                // update game dev project to info or another when you get tired of debug
                filter: "wgpu=error,baba_yaga=debug".to_string(),
                ..default()
            })
            .set(view::window_plugin())
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(PhysicsDebugPlugin)
    .insert_gizmo_config(
        PhysicsGizmos::default(),
        GizmoConfig {
            enabled: false,
            ..default()
        },
    )
    // Enable system ambiguity detection
    .edit_schedule(Update, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Warn,
            ..default()
        });
    })
    .add_systems(
        Update,
        (
            handle_debug_input
                .in_set(InGameSystems::PlayerInput)
                .ambiguous_with_all(),
            (
                view::camera_debug_system,
                debug_vision,
                spawn_enemy_ai_labels,
                update_enemy_ai_labels,
                debug_target_lines,
            )
                .in_set(InGameSystems::HudOverlay)
                .run_if(resource_exists::<DebugRenderEnabled>),
            cleanup_enemy_ai_labels
                .in_set(InGameSystems::HudOverlay)
                .run_if(not(resource_exists::<DebugRenderEnabled>)),
        ),
    );

    app.add_systems(Update, log_transitions::<AppState>);
    app.add_systems(Update, log_transitions::<Menu>);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(FpsOverlayPlugin::default()).configure_sets(
        Update,
        // The overlay only touches its own text and nodes.
        (FpsOverlaySystems::Customize, FpsOverlaySystems::UpdateText)
            .ambiguous_with(InGameSystems::HudOverlay),
    );
}

#[derive(Resource)]
struct DebugRenderEnabled;

fn handle_debug_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
    debug_enabled: Option<Res<DebugRenderEnabled>>,
) {
    if keyboard_input.just_pressed(KeyCode::Comma) {
        if debug_enabled.is_some() {
            commands.remove_resource::<DebugRenderEnabled>();
        } else {
            commands.insert_resource(DebugRenderEnabled);
        }
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}

/// Draws debug gizmos for AI vision direction and cone angles.
fn debug_vision(mut gizmos: Gizmos, query: Query<(&Transform, &Vision)>) {
    for (transform, vision) in &query {
        let origin = transform.translation.xy();
        let forward = vision.aim_direction;

        gizmos.arrow_2d(origin, origin + forward * 64.0, GREEN);

        let left = forward.rotate(Vec2::from_angle(45f32.to_radians()));
        let right = forward.rotate(Vec2::from_angle(-45f32.to_radians()));

        gizmos.line_2d(origin, origin + left * 64.0, YELLOW);
        gizmos.line_2d(origin, origin + right * 64.0, YELLOW);
    }
}

/// Draws a line from each targeting enemy toward its current target position.
/// Green means a clear ray, red means the ray is blocked.
fn debug_target_lines(
    mut gizmos: Gizmos,
    enemies: Query<(&Transform, &TargetInfo, Has<Targeting>), With<Enemy>>,
) {
    for (transform, target_info, has_target) in &enemies {
        if !has_target {
            continue;
        }
        let origin = transform.translation.xy();
        let color = if target_info.line_of_sight {
            GREEN
        } else {
            RED
        };
        gizmos.line_2d(origin, target_info.target_position, color);
    }
}

/// Marker on the floating [`Text2d`] above a debug-tracked enemy.
#[derive(Component)]
struct EnemyAiLabel;

/// Marker on enemies that already have a label child (and a [`ScoreTrace`]),
/// so spawn runs once.
#[derive(Component)]
struct HasAiLabel;

/// Height of the floating label above the enemy's center, in world units.
const AI_LABEL_HEIGHT: f32 = 30.0;

/// Spawns one floating label per enemy while the debug overlay is enabled,
/// and starts tracing its brain's scores.
fn spawn_enemy_ai_labels(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<HasAiLabel>)>,
) {
    for enemy in &enemies {
        commands
            .entity(enemy)
            .insert((HasAiLabel, ScoreTrace::default()));
        commands.entity(enemy).with_child((
            EnemyAiLabel,
            Text2d::new("..."),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor::from(Color::WHITE),
            // Grows upward, so a long score trace stays clear of the enemy.
            Anchor::BOTTOM_CENTER,
            Transform::from_translation(Vec3::new(0.0, AI_LABEL_HEIGHT, ZLayer::AboveSprite.z())),
        ));
    }
}

/// Refreshes each label in plain words: the motion, the target distance
/// in world units, whether the enemy sees you and is attacking, and the score
/// trace of its last decision.
fn update_enemy_ai_labels(
    enemies: Query<
        (
            &Brain,
            &TargetInfo,
            Has<Targeting>,
            Option<&AttackState>,
            Option<&Dash>,
            Option<&Wander>,
            Option<&ScoreTrace>,
        ),
        With<Enemy>,
    >,
    mut labels: Query<(&ChildOf, &mut Text2d), With<EnemyAiLabel>>,
) {
    for (parent, mut text) in &mut labels {
        let Ok((brain, target_info, has_target, attack_state, dash, wander, score_trace)) =
            enemies.get(parent.parent())
        else {
            continue;
        };
        let attacking = attack_state.is_some_and(|attack| attack.is_attacking);
        let mode = short_activity(brain, dash, wander);
        if has_target {
            let sight = if target_info.line_of_sight {
                "sees you"
            } else {
                "blocked"
            };
            text.0 = if attacking {
                format!(
                    "{mode}\ntarget {:.0}\n{sight}, attacking",
                    target_info.distance
                )
            } else {
                format!("{mode}\ntarget {:.0}\n{sight}", target_info.distance)
            };
        } else if attacking {
            text.0 = format!("{mode}\nattacking");
        } else {
            text.0 = mode.to_string();
        }
        if let Some(score_trace) = score_trace {
            append_score_trace(&mut text.0, brain, score_trace);
        }
    }
}

/// Appends one line per behavior from the last decision's score trace.
fn append_score_trace(label: &mut String, brain: &Brain, score_trace: &ScoreTrace) {
    let format_value = |value: Option<f32>| {
        // A missing or reachless capability reads as the largest fraction.
        value.map_or_else(
            || "none".to_string(),
            |value| {
                if value >= f32::MAX {
                    "far".to_string()
                } else {
                    format!("{value:.2}")
                }
            },
        )
    };
    for (behavior, explanation) in brain
        .definition()
        .behaviors
        .iter()
        .zip(&score_trace.behavior_explanations)
    {
        let _ = write!(
            label,
            "
{}",
            behavior.motion.map_or("-", Motion::name)
        );
        if let Some(action) = behavior.action {
            let _ = write!(label, " + {action:?}");
        }
        if let Some(reason) = explanation.skipped {
            let _ = write!(label, ": {}", reason.label());
            continue;
        }
        let _ = write!(label, ": {}", format_value(explanation.score));
        if explanation.momentum > 1.0 {
            let _ = write!(label, " (x{:.2})", explanation.momentum);
        }
        for consideration in &explanation.considerations {
            let _ = write!(
                label,
                " | {} {} [{:?} = {}]",
                consideration.name,
                format_value(consideration.score),
                consideration.input,
                format_value(consideration.raw),
            );
        }
    }
}

/// Removes floating labels and score traces once the debug overlay is toggled off.
fn cleanup_enemy_ai_labels(
    mut commands: Commands,
    labels: Query<Entity, With<EnemyAiLabel>>,
    marked: Query<Entity, With<HasAiLabel>>,
) {
    for label in &labels {
        commands.entity(label).despawn();
    }
    for enemy in &marked {
        commands.entity(enemy).remove::<(HasAiLabel, ScoreTrace)>();
    }
}

/// One-word name for what the enemy is doing: its current motion, with a
/// dash's own policy name and an enemy's wander or rest beat.
fn short_activity(brain: &Brain, dash: Option<&Dash>, wander: Option<&Wander>) -> &'static str {
    match brain.motion() {
        None => "Stopped",
        Some(Motion::Dash) => dash.map_or(Motion::Dash.name(), Dash::name),
        Some(Motion::Wander) => {
            if wander.is_some_and(Wander::is_walking) {
                "Wander"
            } else {
                "Rest"
            }
        }
        Some(motion) => motion.name(),
    }
}
