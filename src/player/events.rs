use bevy::prelude::*;

#[derive(Event)]
pub struct PlayerLevelUpEvent {
    pub new_level: u32,
}

#[derive(Event)]
pub struct PlayerMovementEvent {
    pub direction: Vec2,
}

#[derive(Event)]
pub struct PlayerStoppedEvent;
//This should have a field that tells us the direction the player was facing
//But that comes later

#[derive(Event)]
pub struct MainHandActivated;

#[derive(Event)]
pub struct ResetPlayerPosition;

#[derive(Event)]
pub struct AttemptInteractionInput;
