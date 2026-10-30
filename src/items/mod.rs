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
        StopUsingEquipment, Unequip, tick_equippable_cooldown,
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
    pub use super::{Item, ItemType};
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
    pub item_type: ItemType,
    pub drop_glow_effect: f32,
    pub drop_rotation_timer: f32,
    pub drop_rate: f32,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            value: 10,
            item_type: ItemType::Melee,
            drop_glow_effect: 0.0,
            drop_rotation_timer: 0.0,
            drop_rate: 0.0,
        }
    }
}

impl Item {
    pub fn new(value: u32, item_type: ItemType) -> Self {
        Self {
            value,
            item_type,
            drop_rate: 1.2,
            ..default()
        }
    }
}

#[derive(Clone, Copy)]
pub enum ItemType {
    Melee,
    Staff,
    Potion,
    Tome,
}
