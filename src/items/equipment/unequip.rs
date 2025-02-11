use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::items::inventory::Inventory;

use super::{equippable::Equipped, Equippable};

#[derive(Event)]
pub struct UnequipEvent {
    pub item_entity: Entity,
}

pub fn on_unequip_event(unequip_trigger: Trigger<UnequipEvent>, mut commands: Commands) {
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
    let item_entity = trigger.entity();

    let Ok((equippable, holder, mut visibility)) = item_query.get_mut(item_entity) else {
        info!("Item was despawned prior to unequip");
        return;
    };

    let Ok(mut inventory) = holder_query.get_mut(holder.get()) else {
        info!("Holder was despawned prior to unequip");
        return;
    };

    *visibility = Visibility::Hidden;

    commands.entity(item_entity).remove::<Collider>();

    inventory.unequip(equippable.slot);

    info!("Item Unequipped!");
}
