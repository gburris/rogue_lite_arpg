use bevy::prelude::*;

use crate::prelude::*;

mod consumable;
mod equipment;
mod inventory;
mod lootable;
mod magnet;
mod melee;
mod shield;
mod staff;
mod tome;

pub mod prelude {
    pub use super::consumable::{Consumable, Consume, health_potion};
    pub use super::equipment::prelude::{
        DirectionalPoses, Equip, EquipmentInUse, EquipmentPose, EquipmentRead, EquipmentSlot,
        EquipmentUsage, EquipmentUseFailed, EquipmentUseFailure, EquipmentUseFinished,
        EquipmentUseSucceeded, Equippable, Equipped, Mainhand, Offhand, StartUsingEquipment,
        StopUsingEquipment, Unequip,
    };
    pub use super::inventory::{ItemCapacity, ItemOf, Items};
    pub use super::lootable::DropItem;
    pub use super::magnet::Magnet;
    pub use super::melee::prelude::{axe, freeze_axe, sword};
    pub use super::shield::{ProjectileReflection, knight_shield, magic_shield};
    pub use super::staff::{
        STAFF_REACH, fire_staff, ice_staff, mage_fire_staff_fan, mage_fire_staff_single_bolt,
    };
    pub use super::tome::tome_of_healing;
    pub use super::{Item, ItemKind};
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        equipment::plugin,
        lootable::plugin,
        melee::plugin,
        shield::plugin,
        staff::plugin,
    ));

    app.add_systems(
        FixedUpdate,
        magnet::update_magnet_locations.in_set(MainSystems::InGame),
    )
    .add_observer(inventory::on_item_added_to_inventory)
    .add_observer(consumable::on_consume);
}

/// This is the base component for all items in the game. If you have a new concept that will be
/// shared by all items, add it as a field here.
///
/// Ex.  All items can be dropped, so drop-related info can go here.
#[derive(Component)]
#[require(Visibility::Hidden)]
pub struct Item {
    pub value: u32,
    pub kind: ItemKind,
    pub drop_rate: f32,
}

impl Item {
    pub fn new(value: u32, kind: ItemKind) -> Self {
        Self {
            value,
            kind,
            drop_rate: 1.2,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ItemKind {
    Melee,
    Staff,
    Potion,
    Tome,
}
