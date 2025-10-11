use bevy::prelude::*;

use crate::prelude::*;

#[derive(Message)]
pub struct PlayerMovementEvent {
    pub direction: Vec2,
}

#[derive(Event)]
pub struct PlayerStoppedEvent;

pub fn player_movement(
    player_motion_query: Single<&mut SimpleMotion, With<Player>>,
    mut player_movement_messages: MessageReader<PlayerMovementEvent>,
) {
    let mut motion = player_motion_query.into_inner();
    for player_movement in player_movement_messages.read() {
        motion.start_moving(player_movement.direction);
    }
}

pub fn on_player_stopped(
    _: On<PlayerStoppedEvent>,
    mut player_motion: Single<&mut SimpleMotion, (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    player_motion.stop_moving();
}
