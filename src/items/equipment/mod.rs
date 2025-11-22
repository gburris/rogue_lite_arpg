use core::fmt;

use bevy::prelude::*;

mod equip;
mod equipment_transform;
mod use_equipped;

use crate::prelude::*;

pub mod prelude {
    pub use super::equip::*;
    pub use super::equipment_transform::*;
    pub use super::use_equipped::*;
    pub use super::{
        EquipmentSlot, Equippable, Equipped, Mainhand, MainhandOf, Offhand, OffhandOf,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(equip::plugin);

    app.add_systems(
        Update,
        (
            // Always run this system InGame and InMenu so weapon transforms update as inventory is interacted with
            equipment_transform::update_equipment_transforms.in_set(MainSystems::Shared),
            use_equipped::tick_equippable_use_rate.in_set(InGameSystems::Simulation),
        ),
    );

    app.add_observer(use_equipped::on_ai_equipment_used);
}

#[derive(Component, Clone, Debug)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub equip_type: EquipmentType,
    pub use_rate: Timer, // swing a sword, shoot a weapon, etc...
}

impl Default for Equippable {
    fn default() -> Self {
        Self {
            slot: EquipmentSlot::Mainhand,
            equip_type: EquipmentType::Sword,
            use_rate: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}

impl Equippable {
    pub fn new(slot: EquipmentSlot, equip_type: EquipmentType) -> Self {
        Equippable {
            slot,
            equip_type,
            ..default()
        }
    }
    pub fn from(duration: f32, slot: EquipmentSlot, equip_type: EquipmentType) -> Self {
        Equippable {
            use_rate: Timer::from_seconds(duration, TimerMode::Once),
            slot,
            equip_type,
        }
    }
}

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
        write!(f, "{variant_name}")
    }
}

#[derive(Component, Clone, Debug)]
pub struct Equipped;

#[derive(Component, Clone)]
#[relationship(relationship_target = Mainhand)]
pub struct MainhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = MainhandOf)]
pub struct Mainhand(Entity);

impl Mainhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = Offhand)]
pub struct OffhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OffhandOf)]
pub struct Offhand(Entity);

impl Offhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum EquipmentType {
    Sword,
    Axe,
    Staff,
    Shield,
    Spellbook,
}
