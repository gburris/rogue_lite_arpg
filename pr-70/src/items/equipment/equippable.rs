use std::fmt;

use bevy::prelude::*;

/// Goes on the equipment marking where it should be equipped
///
/// Note: We pass this by value a lot, don't add data to it without consideration for passing this by reference
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EquipmentSlot {
    Mainhand,
    Offhand,
}

impl fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant_name = match *self {
            EquipmentSlot::Mainhand => "Main Hand",
            EquipmentSlot::Offhand => "Off Hand",
        };
        write!(f, "{}", variant_name)
    }
}

#[derive(Component, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub use_rate: Timer, // swing a sword, shoot a weapon, etc...
}

impl Default for Equippable {
    fn default() -> Self {
        Self {
            slot: EquipmentSlot::Mainhand,
            use_rate: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}

impl Equippable {
    pub fn new(slot: EquipmentSlot) -> Self {
        Equippable { slot, ..default() }
    }
    pub fn from(duration: f32, slot: EquipmentSlot) -> Self {
        Equippable {
            use_rate: Timer::from_seconds(duration, TimerMode::Once),
            slot,
        }
    }
}

/// Marker component that represents when an "Equippable" item has been equipped
/// It holds a reference to the entity that has it equipped
///
/// TODO: Bevy 0.16 - Make this a relationship and immutable
#[derive(Component)]
pub struct Equipped {
    /// Entity that has this item equipped
    equipped_to: Entity,
}

impl Equipped {
    pub fn new(equipped_to: Entity) -> Self {
        Self { equipped_to }
    }

    pub fn get_equipped_to(&self) -> Entity {
        self.equipped_to
    }
}
