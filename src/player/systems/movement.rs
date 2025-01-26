use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    map::resources::MapBounds,
    movement::components::{IsMoving, SimpleMotion},
    player::{
        movement::MovementDirection, resources::PlayerSize, Player, PlayerMovementEvent,
        PlayerStoppedEvent, ResetPlayerPosition,
    },
};

// System to handle player movement based on movement events
pub fn player_movement(
    mut player_motion_query: Query<
        (&mut MovementDirection, &mut IsMoving, &mut SimpleMotion),
        With<Player>,
    >,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    for event in event_reader.read() {
        for (mut movement_direction, mut is_moving, mut motion) in player_motion_query.iter_mut() {
            motion.direction = event.direction;
            //Only update the players movement direction value if it's different from the current one
            movement_direction.set_if_neq(MovementDirection::from_vec2(event.direction));
            is_moving.0 = true;
        }
    }
}

pub fn on_player_stopped(
    _: Trigger<PlayerStoppedEvent>,
    mut animation_query: Query<(&mut MovementDirection, &mut IsMoving), With<Player>>,
) {
    let (mut current_player_movement, mut is_moving) = animation_query.single_mut();
    is_moving.0 = false;
    *current_player_movement = MovementDirection::None;
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
