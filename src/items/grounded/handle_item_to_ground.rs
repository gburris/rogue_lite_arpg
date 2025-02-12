use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    items::{inventory::Inventory, Grounded},
    labels::layer::ZLayer,
};

pub fn handle_item_ground_transition(
    trigger: Trigger<OnAdd, Grounded>,
    mut commands: Commands,
    item_query: Query<&Parent>,
    parent_query: Query<&Transform, With<Inventory>>,
) {
    let item_entity = trigger.entity();

    let Ok(parent) = item_query.get(item_entity) else {
        warn!("Grounded item missing parent, maybe unequipped on accident?");
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
