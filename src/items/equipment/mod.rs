use core::fmt;

use bevy::prelude::*;

mod equip;
mod equipment_transform;
mod unequip;
mod use_equipped;

// Only expose the things outside modules need!!!

// Components!!
pub use equipment_transform::EquipmentTransform;

// Equipment events to "get shit done"
pub use use_equipped::EquipmentUseFailed;
pub use use_equipped::EquipmentUseFailure;
pub use use_equipped::StopUsingHoldableEquipmentInput;
pub use use_equipped::UseEquipment;
pub use use_equipped::UseEquipmentInput;

// Observers to be added to respective equipment/weapons that want this functionality
pub use use_equipped::on_equipment_activated;
pub use use_equipped::on_equipment_deactivated;
pub use use_equipped::on_healing_tome_cast;
pub use use_equipped::on_shield_block;
pub use use_equipped::on_weapon_fired;
pub use use_equipped::on_weapon_melee;

pub use unequip::Unequip;

use crate::labels::sets::InGameSystems;
use crate::labels::sets::MainSystems;

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Always run this system InGame and InMenu so weapon transforms update as inventory is interacted with
                equipment_transform::update_equipment_transforms.in_set(MainSystems::Shared),
                use_equipped::tick_equippable_use_rate.in_set(InGameSystems::Simulation),
            ),
        )
        .add_observer(equip::on_item_equipped)
        .add_observer(unequip::on_item_unequipped);
    }
}

#[derive(Component, Clone, Debug)]
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
pub struct MainhandOf(Entity);

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
pub struct OffhandOf(Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OffhandOf)]
pub struct Offhand(Entity);

impl Offhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}
