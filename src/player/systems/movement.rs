use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    animation::MovementDirection,
    map::WorldSpaceConfig,
    movement::components::{IsMoving, SimpleMotion},
    player::{
        resources::PlayerSize, Player, PlayerMovementEvent, PlayerStoppedEvent, ResetPlayerPosition,
    },
};

// System to handle player movement based on movement events
pub fn player_movement(
    player_motion_query: Single<
        (&mut MovementDirection, &mut IsMoving, &mut SimpleMotion),
        With<Player>,
    >,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    let (mut movement_direction, mut is_moving, mut motion) = player_motion_query.into_inner();
    for event in event_reader.read() {
        motion.direction = event.direction;
        movement_direction.set_if_neq(MovementDirection::from_vec2(event.direction));
        is_moving.0 = true;
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
// System to keep player within map bounds
pub fn enforce_map_bounds(
    mut query: Query<&mut Transform, With<Player>>,
    world_config: Res<WorldSpaceConfig>,
    playersize: Res<PlayerSize>,
) {
    let world_min_x = world_config.world_origin.x
        - (world_config.map_size.x as f32 * world_config.tile_size.x) / 2.0;
    let world_max_x = world_config.world_origin.x
        + (world_config.map_size.x as f32 * world_config.tile_size.x) / 2.0;
    let world_min_y = world_config.world_origin.y
        - (world_config.map_size.y as f32 * world_config.tile_size.y) / 2.0;
    let world_max_y = world_config.world_origin.y
        + (world_config.map_size.y as f32 * world_config.tile_size.y) / 2.0;

    for mut transform in query.iter_mut() {
        transform.translation.x = transform.translation.x.clamp(
            world_min_x + playersize.x / 2.0,
            world_max_x - playersize.x / 2.0,
        );
        transform.translation.y = transform.translation.y.clamp(
            world_min_y + playersize.y / 2.0,
            world_max_y - playersize.y / 2.0,
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
