use crate::{
    configuration::GameCollisionLayer,
    despawn::components::LiveDuration,
    items::{Grounded, GroundedItemEffect, ItemToGroundEvent},
};
use avian2d::prelude::{Collider, CollisionLayers, Sensor};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

pub fn handle_item_ground_transition(
    item_drop_trigger: Trigger<ItemToGroundEvent>,
    mut commands: Commands,
) {
    let mut rng = thread_rng();
    warn!("handle_item_ground_transition");
    let offset = Vec3::new(rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0), 3.0);
    commands
        .entity(item_drop_trigger.entity())
        .remove_parent()
        .insert(LiveDuration::new(10.0))
        .insert(Collider::circle(10.0))
        .insert(Transform::from_translation(
            item_drop_trigger.origin_position + offset,
        ))
        .insert(Sensor)
        .insert(Visibility::Visible)
        .insert(GroundedItemEffect::default())
        .insert(CollisionLayers::new(
            GameCollisionLayer::Interaction,
            [GameCollisionLayer::Player],
        ))
        .insert(Grounded);
}
