use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    configuration::ZLayer,
    items::{equipment::Equipped, inventory::Inventory, Item, ItemDropEvent, Lootable},
    player::interact::InteractionZone,
};

/// Notes:
/// 1. ItemDropEvent is for items only!
/// 2. This event will handle unequipping and removing any items dropped from the inventory of the holder
/// 3. Needs parent to be holder for position, then removes parent
pub fn handle_item_ground_transition(
    trigger: Trigger<ItemDropEvent>,
    mut commands: Commands,
    item_query: Query<&Parent, With<Item>>,
    mut parent_query: Query<(&Transform, &mut Inventory)>,
) {
    let item_entity = trigger.entity();

    let Ok(parent) = item_query.get(item_entity) else {
        warn!("Lootable item missing parent");
        return;
    };

    let Ok((parent_transform, mut inventory)) = parent_query.get_mut(parent.get()) else {
        error!("Why does the parent not have a transform or inventory on drop");
        return;
    };

    let mut rng = thread_rng();
    let offset = Vec2::new(rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0));
    let final_position =
        (parent_transform.translation.truncate() + offset).extend(ZLayer::OnGround.z());

    // We don't care if item is actually found in inventory
    inventory.remove_item(item_entity).ok();

    trace!("Dropping item at {}", offset);

    commands
        .entity(item_entity)
        .remove::<Equipped>()
        .insert((
            Lootable,
            Visibility::Visible,
            Transform::from_translation(final_position),
        ))
        .remove_parent()
        .with_child(InteractionZone::ITEM_PICKUP);
}
