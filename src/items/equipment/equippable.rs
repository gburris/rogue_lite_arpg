use bevy::prelude::*;

use crate::prelude::*;

/// An item that can be equipped and used from a slot. `reach` is the farthest
/// target distance an AI holder uses it at; `None` means AI never aims it.
#[derive(Component, Clone, Debug)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub cooldown: Timer, // swing a sword, shoot a weapon, etc...
    pub poses: &'static DirectionalPoses,
    pub reach: Option<f32>,
}

impl Equippable {
    pub fn new(slot: EquipmentSlot, duration: f32, poses: &'static DirectionalPoses) -> Self {
        Self {
            slot,
            poses,
            cooldown: Timer::from_seconds(duration, TimerMode::Once),
            reach: None,
        }
    }

    /// Lets an AI holder use this item on targets up to `reach` away.
    pub fn with_reach(self, reach: f32) -> Self {
        Self {
            reach: Some(reach),
            ..self
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Equipped;
