use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    map::WorldSpaceConfig,
    movement::components::SimpleMotion,
    player::{
        resources::PlayerSize, Player, PlayerMovementEvent, PlayerStoppedEvent, ResetPlayerPosition,
    },
};

// fires when the player moves
pub fn player_movement(
    player_motion_query: Single<&mut SimpleMotion, With<Player>>,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    let mut motion = player_motion_query.into_inner();
    for event in event_reader.read() {
        motion.direction = event.direction;
        motion.can_move = true;
    }
}

//Fires once the player has stopped moving
pub fn on_player_stopped(
    _: Trigger<PlayerStoppedEvent>,
    mut motion_query: Query<&mut SimpleMotion, With<Player>>,
) {
    let mut motion = motion_query.single_mut();
    motion.can_move = false;
}

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
