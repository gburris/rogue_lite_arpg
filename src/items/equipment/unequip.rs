use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::items::inventory::Inventory;

use super::{equippable::Equipped, Equippable};

#[derive(Event)]
pub struct UnequipEvent {
    pub item_entity: Entity,
}

pub fn on_unequip_event(unequip_trigger: Trigger<UnequipEvent>, mut commands: Commands) {
    info!("On Unequip Event");

    commands
        .entity(unequip_trigger.item_entity)
        .remove::<Equipped>();
}

pub fn on_item_unequipped(
    trigger: Trigger<OnRemove, Equipped>,
    mut commands: Commands,
    mut item_query: Query<(&Equippable, &Parent, &mut Visibility)>,
    mut holder_query: Query<&mut Inventory>,
) {
    info!("On Item Unequipped");
    let item_entity = trigger.entity();

    let Ok((equippable, holder, mut visibility)) = item_query.get_mut(item_entity) else {
        debug!("Item was despawned prior unequip");
        return;
    };

    let mut inventory = holder_query
        .get_mut(holder.get())
        .expect("If item has a parent, that parent should have an inventory");

    *visibility = Visibility::Hidden;

    inventory.unequip(equippable.slot);

    commands
        .entity(item_entity)
        .remove::<Collider>()
        .remove_parent();
}
