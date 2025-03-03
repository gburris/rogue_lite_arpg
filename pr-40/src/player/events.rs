use bevy::prelude::*;

use crate::items::equipment::EquipmentSlot;

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

#[derive(Event)]
pub struct UseEquipmentInputEvent {
    pub slot: EquipmentSlot,
}
