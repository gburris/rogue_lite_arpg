use ::bevy::prelude::*;
use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, Sensor};

use crate::{
    despawn::components::LiveDuration,
    items::{inventory::inventory::Inventory, Autoloot, Grounded},
    player::{AttemptInteractionInput, Player},
};

pub fn on_grounded_item_input_interaction(
    _: Trigger<AttemptInteractionInput>,
    mut commands: Commands,
    colliding_items: Query<(Entity, &CollidingEntities), (With<Grounded>, Without<Autoloot>)>,
    player: Single<(Entity, &mut Inventory), With<Player>>,
) {
    let (player_entity, mut inventory) = player.into_inner();

    for (item_entity, colliding_entities) in colliding_items.iter() {
        if colliding_entities.contains(&player_entity) {
            if inventory.add_item(item_entity).is_ok() {
                commands.entity(player_entity).add_child(item_entity);

                commands
                    .entity(item_entity)
                    .remove::<Grounded>()
                    .remove::<Collider>()
                    .remove::<Sensor>()
                    .remove::<CollidingEntities>()
                    .remove::<LiveDuration>()
                    .remove::<CollisionLayers>()
                    .insert(Visibility::Hidden);
            } else {
                warn!("Inventory is full!")
            }
        }
    }
}
