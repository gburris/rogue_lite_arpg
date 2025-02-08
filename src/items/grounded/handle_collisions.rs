use ::bevy::prelude::*;
use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, Sensor};

use crate::{
    despawn::components::LiveDuration,
    items::{inventory::inventory::Inventory, Grounded, GroundedItemEffect},
    player::Player,
};

pub fn handle_grounded_item_collision(
    mut commands: Commands,
    colliding_items: Query<(Entity, &CollidingEntities), With<Grounded>>,
    mut inventory_query: Query<&mut Inventory, With<Player>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_entity = if let Ok(entity) = player_query.get_single() {
        entity
    } else {
        return;
    };

    for (item_entity, colliding_entities) in colliding_items.iter() {
        if colliding_entities.contains(&player_entity) {
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                if inventory.add_item(item_entity).is_ok() {
                    // Successfully added to inventory, remove ground state
                    commands
                        .entity(item_entity)
                        .remove::<Grounded>()
                        .remove::<Collider>()
                        .remove::<Sensor>()
                        .remove::<GroundedItemEffect>()
                        .remove::<LiveDuration>()
                        .remove::<CollisionLayers>()
                        .insert(Visibility::Hidden);
                }
            }
        }
    }
}

