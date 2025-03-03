use bevy::prelude::*;

use crate::{
    labels::layer::ZLayer,
    player::{events::PlayerLevelUpEvent, Player, PlayerExperience, PlayerLevel},
};

#[derive(Component)]
pub struct LevelUpEffect {
    timer: Timer,
    initial_scale: Vec3,
    rotation_speed: f32,
}

#[derive(Component)]
pub struct LevelUpText;

pub fn on_player_experience_change(
    mut commands: Commands,
    mut player_query: Query<
        (&mut PlayerExperience, &PlayerLevel),
        (Changed<PlayerExperience>, With<Player>),
    >,
) {
    if let Ok((mut exp, player_level)) = player_query.get_single_mut() {
        while exp.current >= exp.next_level_requirement {
            exp.current -= exp.next_level_requirement;
            exp.next_level_requirement *= 2;

            commands.trigger(PlayerLevelUpEvent {
                new_level: player_level.current + 1,
            });
        }
    }
}

pub fn on_level_up(
    trigger: Trigger<PlayerLevelUpEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Single<(Entity, &mut PlayerLevel), With<Player>>,
) {
    let (player_entity, mut player_level) = player_query.into_inner();
    player_level.current = trigger.new_level;

    // Spawn circular ring effect
    let ring = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(1.0, 0.9, 0.0, 0.7)))),
            Transform::from_xyz(0.0, 0.0, ZLayer::LevelUpEffect.z()),
            LevelUpEffect {
                timer: Timer::from_seconds(1.2, TimerMode::Once),
                initial_scale: Vec3::splat(0.1),
                rotation_speed: 2.0,
            },
        ))
        .id();

    // Spawn level up text
    let text = commands
        .spawn((
            Text2d::new("Level up!"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor::from(Color::srgb(1.0, 0.84, 0.0)),
            Transform::from_xyz(0.0, 60.0, ZLayer::LevelUpEffect.z()),
            LevelUpText,
        ))
        .id();

    // Add both effects as children of the player
    commands.entity(player_entity).add_children(&[ring, text]);
}

pub fn animate_level_up(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &mut LevelUpEffect,
            &Parent,
        ),
        Without<LevelUpText>,
    >,
    mut text_query: Query<(Entity, &mut Transform, &Parent), With<LevelUpText>>,
) {
    // Animate ring effect
    for (entity, mut transform, mut material, mut effect, _parent) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());
        let progress = effect.timer.fraction();

        // Scale up and rotate
        transform.scale = effect.initial_scale * (1.0 + progress * 4.0);
        transform.rotate_z(effect.rotation_speed * time.delta_secs());

        // Fade out
        if let Some(material) = material.get_field_mut::<ColorMaterial>(0) {
            material.color.set_alpha(1.0 - progress);
        }

        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }

    // Animate text
    for (entity, mut transform, _parent) in text_query.iter_mut() {
        transform.translation.y += 50.0 * time.delta_secs();

        // Remove text after 1 second
        if transform.translation.y > 120.0 {
            commands.entity(entity).despawn();
        }
    }
}
