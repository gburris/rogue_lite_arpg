use crate::components::{Enemy, HealthBar, HealthText};
use bevy::prelude::*;

pub fn spawn_health_displays(
    mut commands: Commands,
    query: Query<(Entity, &Enemy), Added<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, enemy) in query.iter() {
        // Spawn the health bar
        commands.spawn((
            HealthBar { owner: entity },
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(50.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 30.0, 1.0), // Offset above enemy
        ));
    }
}

// Update the health displays
pub fn update_health_displays(
    mut bar_query: Query<(&mut Transform, &mut Sprite, &HealthBar)>,
    enemy_query: Query<(&Transform, &Enemy), Without<Transform>>, // Exclude Transform from the enemy query
) {
    // Update health bars
    for (mut bar_transform, mut sprite, health_bar) in bar_query.iter_mut() {
        if let Ok((enemy_transform, enemy)) = enemy_query.get(health_bar.owner) {
            // Update position to follow enemy
            bar_transform.translation = enemy_transform.translation + Vec3::new(0.0, 30.0, 0.1);

            // Update health bar width based on health percentage
            let health_percent = (enemy.health / 25.0).clamp(0.0, 1.0); // Assuming max health is 25
            sprite.custom_size = Some(Vec2::new(50.0 * health_percent, 5.0));
        }
    }
}

// Cleanup health displays when enemy is destroyed
pub fn cleanup_health_displays(
    mut commands: Commands,
    enemy_query: Query<(), With<Enemy>>,
    bar_query: Query<(Entity, &HealthBar)>,
    text_query: Query<(Entity, &HealthText)>,
) {
    // Remove health bars for destroyed enemies
    for (bar_entity, health_bar) in bar_query.iter() {
        if enemy_query.get(health_bar.owner).is_err() {
            commands.entity(bar_entity).despawn();
        }
    }
}
