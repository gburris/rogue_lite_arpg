use std::f32::consts::PI;

use bevy::{prelude::*, transform};
use rand::{thread_rng, Rng};

use crate::{
    combat::{
        attributes::{Health, Mana},
        damage::events::DamageDealtEvent,
    },
    despawn::components::LiveDuration,
    labels::layer::ZLayer,
    player::{components::Player, PlayerExperience, PlayerLevel},
};

#[derive(Component)]
pub struct GameOverlay;

#[derive(Component)]
pub struct PlayerOverlayStatsText;

#[derive(Component)]
pub struct ManaBar;

const RED_COLOR: bevy::prelude::Color = Color::srgb(1.0, 0.0, 0.0);

pub fn spawn(mut commands: Commands) {
    commands
        .spawn((
            GameOverlay,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(20.0), Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerOverlayStatsText,
                Node {
                    height: Val::Px(80.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Text::new("(0.0, 0.0)"),
            ));

            // Spacer between stats text and mana bar
            parent.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            });

            // Footer
            parent
                .spawn((Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    ..default()
                },))
                .with_children(|footer| {
                    footer
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(15.0),
                                ..default()
                            },
                            BackgroundColor::from(Color::srgb(0.21, 0.21, 0.21)),
                        ))
                        .with_child((
                            ManaBar,
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(15.0),
                                ..default()
                            },
                            BackgroundColor::from(Color::srgb(0.0, 0.173, 0.878)),
                        ));
                });
        });
}

pub fn update(
    player: Single<(&PlayerExperience, &PlayerLevel, &Health, &Mana), With<Player>>,
    overlay_stat_text: Single<&mut Text, With<PlayerOverlayStatsText>>,
) {
    let (exp, level, health, mana) = player.into_inner();

    let mut overlay_stat_text = overlay_stat_text.into_inner();
    *overlay_stat_text = Text::new(format!(
        "Level: {:.1} Exp: {:.1} / {:.1} ||| Health: {:.1} / {:.1} ||| Mana: {:.1} / {:.1}",
        level.current,
        exp.current,
        exp.next_level_requirement,
        health.hp,
        health.max_hp,
        mana.current_mana,
        mana.max_mana
    ));
}

pub fn update_mana_bar(
    player_mana: Single<&Mana, (With<Player>, Changed<Mana>)>,
    mana_bar: Single<&mut Node, With<ManaBar>>,
) {
    let mana = player_mana.into_inner();
    let mut mana_bar = mana_bar.into_inner();

    // Mana bar gets longer as player gets higher max mana
    let max_mana_bar_length = mana.max_mana * 4.0;

    mana_bar.width = Val::Px((mana.current_mana / mana.max_mana) * max_mana_bar_length);
}

pub fn on_damage_overlay_amount(
    damage_trigger: Trigger<DamageDealtEvent>,
    mut commands: Commands,
    damaged_query: Query<&Transform>,
) {
    if let Ok(transform) = damaged_query.get(damage_trigger.entity()) {
        // Create a quaternion for the random rotation
        let random_rotation = Quat::from_axis_angle(Vec3::Z, random_angle(30.0));

        // Combine the original rotation with the random offset
        let new_rotation = random_rotation * transform.rotation;

        // Get rotation assuming sprite is facing "UP" (y axis)
        let rotated_vector = (new_rotation * Vec3::Y).truncate();

        // Scale the direction vector by a static offset value
        let static_offset = 80.0; // Example offset value
        let scaled_offset = rotated_vector.normalize() * static_offset;

        commands.entity(damage_trigger.entity()).with_child((
            Text2d::new(damage_trigger.damage.to_string()),
            TextColor::from(RED_COLOR),
            LiveDuration(Timer::from_seconds(0.6, TimerMode::Once)),
            Transform::from_translation(scaled_offset.extend(ZLayer::VisualEffect.z())),
        ));
    }
}

// Generate a random angle between -angle_range and angle_range degrees (convert to radians)
fn random_angle(angle_range: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(-angle_range..angle_range).to_radians()
}

// trace!(
//     "Spawning damage text with random rotation: {}, enemy rotation: {}, new rotation: {}, scaled offset: {}",
//     random_rotation, new_rotation, rotated_vector, scaled_offset
// );
