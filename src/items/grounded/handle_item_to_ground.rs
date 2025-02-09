use crate::{
    combat::melee::components::ActiveMeleeAttack,
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
    let offset = Vec3::new(rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0), 3.0);
    let final_position = item_drop_trigger.origin_position + offset;

    // First, reset everything about the transform
    // Needs to be two seperate "Command" operations
    // Otherwise transforms get messed up for equipped items
    commands
        .entity(item_drop_trigger.entity())
        .remove_parent()
        .remove::<ActiveMeleeAttack>()
        .remove::<Sensor>()
        .remove::<Collider>()
        .insert(Transform::default());

    commands
        .entity(item_drop_trigger.entity())
        .insert(Transform::from_translation(final_position))
        .insert(LiveDuration::new(10.0))
        .insert(Collider::circle(10.0))
        .insert(Sensor)
        .insert(Visibility::Visible)
        .insert(GroundedItemEffect::default())
        .insert(CollisionLayers::new(
            GameCollisionLayer::Interaction,
            [GameCollisionLayer::Player],
        ))
        .insert(Grounded);
}
