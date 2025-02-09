use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    map::WorldSpaceConfig,
    movement::components::SimpleMotion,
    npc::NPC,
    player::{resources::PlayerSize, Player, PlayerMovementEvent, PlayerStoppedEvent},
};

pub fn player_movement(
    player_motion_query: Single<&mut SimpleMotion, With<Player>>,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    let mut motion = player_motion_query.into_inner();
    for event in event_reader.read() {
        motion.start_moving(event.direction);
    }
}

pub fn on_player_stopped(
    _: Trigger<PlayerStoppedEvent>,
    mut player_motion: Single<&mut SimpleMotion, (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    player_motion.stop_moving();
}

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
