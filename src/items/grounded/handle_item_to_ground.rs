use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    items::{inventory::Inventory, Grounded},
    labels::layer::ZLayer,
};

/// Notes:
/// 1. Grounded is for ITEMS. If it isn't an item, it can't be grounded in the current state
/// 2. This item should not exist in the entities inventory anymore,
/// 3. Call remove::<Equipped>() FIRST
/// 4. Still needs parent to be holder for position, then removes parent
///
/// This IS brittle, and will be made so much easier in Bevy 0.16 with relations circa end of March
pub fn handle_item_ground_transition(
    trigger: Trigger<OnAdd, Grounded>,
    mut commands: Commands,
    item_query: Query<&Parent>,
    parent_query: Query<&Transform, With<Inventory>>,
) {
    let item_entity = trigger.entity();

    let Ok(parent) = item_query.get(item_entity) else {
        warn!("Grounded item missing parent");
        return;
    };

    let Ok(parent_transform) = parent_query.get(parent.get()) else {
        error!("Why does the parent not have a transform or inventory on drop");
        return;
    };

    let mut rng = thread_rng();
    let offset = Vec2::new(rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0));
    let final_position =
        (parent_transform.translation.truncate() + offset).extend(ZLayer::ItemOnGround.z());

    warn!("Dropping item at {}", offset);

    commands
        .entity(item_entity)
        .insert(Transform::from_translation(final_position))
        .insert(Visibility::Visible)
        .remove_parent();
}
