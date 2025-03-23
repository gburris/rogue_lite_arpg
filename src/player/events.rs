use bevy::prelude::*;

use crate::items::equipment::EquipmentSlot;

#[derive(Event)]
pub struct PlayerLevelUpEvent;

#[derive(Event)]
pub struct PlayerStoppedEvent;

#[derive(Event)]
pub struct UseEquipmentInputEvent {
    pub slot: EquipmentSlot,
}

#[derive(Event)]
pub struct StopUsingHoldableEquipmentInputEvent {
    pub slot: EquipmentSlot,
}
