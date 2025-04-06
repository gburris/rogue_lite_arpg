use bevy::prelude::*;

use crate::{
    ai::SimpleMotion,
    enemy::Enemy,
    npc::NPC,
    player::{Player, PlayerMovementEvent, PlayerStoppedEvent},
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
