// Bevy class to handle player level up
use bevy::{prelude::*, transform};

use crate::events::PlayerLevelUpEvent;
#[derive(Component)]
pub struct LevelUpAnimation {
    timer: Timer,
    initial_scale: Vec3,
}

pub fn handle_player_level_up(
    mut commands: Commands,
    mut level_up_events: EventReader<PlayerLevelUpEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in level_up_events.read() {
        println!("Player leveled up to level {}", event.new_level);
        commands.spawn((
            Transform::from_translation(event.position + Vec3::new(0.0, 50.0, 10.0)),
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
            commands.entity(entity).despawn();
        }
    }
}
