use avian2d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;

use crate::{
    map::resources::MapBounds,
    movement::components::{IsMoving, SimpleMotion},
    player::{Player, PlayerMovementEvent, ResetPlayerPosition},
    resources::PlayerSize,
};

// System to handle player movement based on movement events
pub fn player_movement(
    mut query: Query<(&mut IsMoving, &mut SimpleMotion), With<Player>>,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    for event in event_reader.read() {
        for (mut is_moving, mut motion) in query.iter_mut() {
            motion.direction = event.direction;
            is_moving.0 = true;
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
