// Bevy class to handle player level up
use bevy::prelude::*;

use crate::{labels::layer::ZLayer, player::{events::PlayerLevelUpEvent, Player, PlayerExperience, PlayerLevel}};
#[derive(Component)]
pub struct LevelUpAnimation {
    timer: Timer,
    initial_scale: Vec3,
}

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
            exp.next_level_requirement *= 2; // Double exp requirement

            commands.trigger(PlayerLevelUpEvent {
                new_level: player_level.current + 1,
            });
        }
    }
}

pub fn on_level_up(
    trigger: Trigger<PlayerLevelUpEvent>,
    mut commands: Commands,
    mut player_query: Query<(&mut PlayerLevel, &Transform), With<Player>>,
) {
    if let Ok((mut player_level, transform)) = player_query.get_single_mut() {
        // TODO: Can probably make this += 1 and remove new_level from level up event
        player_level.current = trigger.new_level;

        commands.spawn((
            Transform::from_translation(
                transform.translation + Vec3::new(0.0, 50.0, ZLayer::VisualEffect.z()),
            ),
            Sprite {
                color: Color::srgb(1.0, 0.9, 0.0), // Gold color
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            LevelUpAnimation {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
                initial_scale: Vec3::splat(1.0),
            },
        ));
    }
}

pub fn animate_level_up(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut LevelUpAnimation)>,
) {
    for (entity, mut transform, mut sprite, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());
        let progress = animation.timer.fraction();

        transform.scale = animation.initial_scale * (1.0 + progress);
        sprite.color.set_alpha(1.0 - progress);

        if animation.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
