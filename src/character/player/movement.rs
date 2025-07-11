use bevy::prelude::*;

use crate::prelude::*;

#[derive(Event)]
pub struct PlayerMovementEvent {
    pub direction: Vec2,
}

#[derive(Event)]
pub struct PlayerStoppedEvent;

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
