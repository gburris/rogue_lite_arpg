use avian2d::prelude::*;
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

use super::Player;

/// Component to be spawned as a child of any entity. When the player walks within "radius" and clicks "interact" (default: Spacebar)
/// this component will trigger the specified `[#InteractionEvent]` on the parent entity (ex. Open Chest, Talk to NPC, Item Pickup)
#[derive(Component)]
#[require(
    Sensor,
    CollidingEntities,
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, GameCollisionLayer::Player))
)]
pub struct InteractionZone {
    pub radius: f32,
}

#[derive(Event)]
pub struct PlayerInteractionInput;

#[derive(Event)]
pub struct InteractionEvent {
    pub interaction_zone_entity: Entity,
}

pub fn on_player_interaction_input(
    _: Trigger<PlayerInteractionInput>,
    mut commands: Commands,
    query: Query<(Entity, &Parent, &CollidingEntities)>,
    player_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_query.into_inner();

    for (interaction_zone_entity, parent, colliding_entities) in query.iter() {
        // Check if any of the colliding entities is the player
        if colliding_entities.contains(&player_entity) {
            commands.trigger_targets(
                InteractionEvent {
                    interaction_zone_entity,
                },
                parent.get(),
            );
        }
    }
}

pub fn on_interaction_zone_added(
    trigger: Trigger<OnAdd, InteractionZone>,
    mut commands: Commands,
    interact_query: Query<&InteractionZone>,
) {
    // We can unwrap since this is an OnAdd. Surely it exists right 0.o
    let interact = interact_query.get(trigger.entity()).unwrap();

    commands
        .entity(trigger.entity())
        .insert(Collider::circle(interact.radius));
}
