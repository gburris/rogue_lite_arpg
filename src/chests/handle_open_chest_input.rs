use avian2d::prelude::CollidingEntities;

use bevy::prelude::*;

use crate::player::{AttemptInteractionInput, Player};

use super::components::{ChestInteractionRadius, OpenChest};

pub fn handle_open_chest_input(
    _: Trigger<AttemptInteractionInput>,
    mut commands: Commands,
    query: Query<(&Parent, &CollidingEntities), With<ChestInteractionRadius>>,
    player_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_query.into_inner();
    for (parent, colliding_entities) in &query {
        // Check if any of the colliding entities is the player
        if colliding_entities.contains(&player_entity) {
            commands.trigger(OpenChest {
                chest_entity: parent.get(), //This is the chest
            });
        }
    }
}
