use bevy::prelude::*;

use crate::prelude::*;

mod consumable;
mod equipment;
mod lootable;
mod magnet;
mod melee;
mod shield;
mod staff;
mod tome;

pub mod prelude {
    pub use super::consumable::{Consumable, Consume, health_potion};
    pub use super::equipment::prelude::{
        DEFAULT_EQUIPMENT_POSES, DirectionalPoses, Equip, EquipmentInUse, EquipmentPose,
        EquipmentRead, EquipmentSlot, EquipmentUsage, EquipmentUseFailed, EquipmentUseFailure,
        EquipmentUseFinished, EquipmentUseSucceeded, Equippable, Equipped, Mainhand, Offhand,
        StartUsingEquipment, StopUsingEquipment, Unequip, tick_equippable_cooldown,
    };
    pub use super::lootable::ItemDrop;
    pub use super::magnet::Magnet;
    pub use super::melee::prelude::{axe, freeze_axe, sword};
    pub use super::shield::{ProjectileReflection, knight_shield, magic_shield};
    pub use super::staff::{
        STAFF_REACH, fire_staff, ice_staff, mage_fire_staff_fan, mage_fire_staff_single_bolt,
    };
    pub use super::tome::tome_of_healing;
    pub use super::{Item, ItemCapacity, ItemOf, ItemType, Items};
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
    .add_observer(on_item_added_to_inventory)
    .add_observer(consumable::on_consume_event);
}

/// This is the base component for all items in the game. If you have a new concept that will be
/// shared by all items, add it as a field here.
///
/// Ex.  All items can be dropped, so drop-related info can go here
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
        Item {
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
        Item {
            value,
            item_type,
            drop_rate: 1.2,
            ..default()
        }
    }
}

/// Relationship linking an item to its holder (inventory or equipment owner).
#[derive(Component, Clone)]
#[relationship(relationship_target = Items)]
pub struct ItemOf {
    /// Entity holding this item.
    pub holder: Entity,
}

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
pub struct Items(Vec<Entity>);

#[derive(Clone, Copy)]
pub enum ItemType {
    Melee,
    Staff,
    Potion,
    Tome,
}

#[derive(Component)]
pub struct ItemCapacity(pub usize);

fn on_item_added_to_inventory(
    item_added: On<Add, ItemOf>,
    mut commands: Commands,
    item_query: Query<&ItemOf>,
    holder_query: Query<(Option<&Items>, Option<&ItemCapacity>)>,
) -> Result {
    let holder_entity = item_query.get(item_added.entity)?;

    let (items, capacity) = holder_query.get(holder_entity.holder)?;

    let capacity = capacity.map_or(usize::MAX, |c| c.0);

    if items.map_or(0, bevy::prelude::RelationshipTarget::len) >= capacity {
        commands.trigger(ItemDrop {
            item: item_added.entity,
        });
    }

    Ok(())
}
