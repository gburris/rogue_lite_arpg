use bevy::prelude::*;

use crate::{
    configuration::ZLayer,
    despawn::components::LiveDuration,
    player::{events::PlayerLevelUpEvent, player_data::PlayerData, Player},
};

#[derive(Component)]
pub struct LevelUpEffect;

#[derive(Component)]
pub struct LevelUpText;

pub fn on_player_experience_change(mut commands: Commands, mut player: Single<&mut Player>) {
    while player.attempt_level_up() {
        commands.trigger(PlayerLevelUpEvent);
    }
}

pub fn on_level_up(
    _: Trigger<PlayerLevelUpEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_data: Res<PlayerData>,
    player_entity: Single<Entity, With<Player>>,
) {
    let PlayerData {
        LEVEL_UP_RING_SIZE,
        LEVEL_UP_ANIMATION_DURATION,
        ..
    } = *player_data;
    commands
        .entity(player_entity.into_inner())
        .with_children(|builder| {
            // Spawn circular ring effect
            builder.spawn((
                LevelUpEffect,
                Mesh2d(meshes.add(Circle::new(LEVEL_UP_RING_SIZE.0))),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from(Color::srgba(1.0, 0.9, 0.0, 0.7))),
                ),
                Transform::from_translation(Vec2::ZERO.extend(ZLayer::BehindSprite.z())),
                LiveDuration::new(LEVEL_UP_ANIMATION_DURATION),
            ));

            // Level up text above player's head
            builder.spawn((
                LevelUpText,
                Text2d::new("Level up!"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor::from(Color::srgb(1.0, 0.84, 0.0)),
                Transform::from_xyz(0.0, 60.0, ZLayer::BehindSprite.z()),
                LiveDuration::new(LEVEL_UP_ANIMATION_DURATION),
            ));
        });
}

pub fn animate_level_up(
    mut effect_query: Query<
        (
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
            &LiveDuration,
        ),
        (With<LevelUpEffect>, Without<LevelUpText>),
    >,
    player_data: Res<PlayerData>,
    mut text_query: Query<(&mut Transform, &LiveDuration), With<LevelUpText>>,
) {
    let PlayerData {
        MAX_RING_SCALE,
        LEVEL_UP_ROTATION_SPEED,
        LEVEL_UP_TEXT_MAX_HEIGHT,
        ..
    } = *player_data;
    // Animate ring effect
    for (mut transform, mut material, duration) in effect_query.iter_mut() {
        let progress = duration.0.fraction();

        // Scale up and rotate
        transform.scale = Vec3::splat(MAX_RING_SCALE * progress);
        transform.rotate_z(LEVEL_UP_ROTATION_SPEED * duration.0.elapsed_secs());

        // Fade out
        if let Some(material) = material.get_field_mut::<ColorMaterial>(0) {
            material.color.set_alpha(1.0 - progress);
        }
    }

    // Animate text
    for (mut transform, duration) in text_query.iter_mut() {
        transform.translation.y = LEVEL_UP_TEXT_MAX_HEIGHT * duration.0.fraction();
    }
}
