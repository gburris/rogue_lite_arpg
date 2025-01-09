use avian2d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;

use crate::{
    components::Speed,
    map::resources::MapBounds,
    player::{Player, PlayerMovementEvent, ResetPlayerPosition},
    resources::PlayerSize,
};

// System to handle player movement based on movement events
pub fn player_movement(
    mut query: Query<(&Player, &Speed, &mut LinearVelocity)>,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    for event in event_reader.read() {
        for (_player, speed, mut linear_velocity) in query.iter_mut() {
            // Update linear velocity based on input direction
            linear_velocity.x = event.direction.x * speed.velocity;
            linear_velocity.y = event.direction.y * speed.velocity;
        }
    }
}

// System to keep player within map bounds
pub fn enforce_map_bounds(
    mut query: Query<&mut Transform, With<Player>>,
    map_bounds: Res<MapBounds>,
    playersize: Res<PlayerSize>,
) {
    for mut transform in query.iter_mut() {
        // Clamp the player position within the map bounds
        transform.translation.x = transform.translation.x.clamp(
            map_bounds.min_x + playersize.x / 2.0,
            map_bounds.max_x - playersize.x / 2.0,
        );
        transform.translation.y = transform.translation.y.clamp(
            map_bounds.min_y + playersize.y / 2.0,
            map_bounds.max_y - playersize.y / 2.0,
        );
    }
}

// Option 2: Add this system to your game to force rotation to zero
pub fn lock_player_rotation(
    mut query: Query<(&mut Transform, &mut AngularVelocity), With<Player>>,
) {
    for (mut transform, mut angular_velocity) in query.iter_mut() {
        // Force rotation to zero
        transform.rotation = Quat::IDENTITY;
        // Prevent any angular velocity
        angular_velocity.0 = 0.0;
    }
}

pub fn reset_player_position(
    _: Trigger<ResetPlayerPosition>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
) {
    if let Ok((mut transform, mut linear_velocity)) = player_query.get_single_mut() {
        // Reset position
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;

        // Reset velocity
        linear_velocity.x = 0.0;
        linear_velocity.y = 0.0;
    }
}
