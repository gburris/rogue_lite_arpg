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
    Collider(|| Collider::circle(100.0)),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, GameCollisionLayer::Player))
)]
pub struct InteractRadius {
    pub event: InteractionEvent,
}

#[derive(Event)]
pub struct InteractInputEvent;

#[derive(Event, Clone)]
pub enum InteractionEvent {
    ChestOpened,
    NpcDialogue,
}

pub fn on_interact_radius_collision(
    _: Trigger<InteractInputEvent>,
    mut commands: Commands,
    query: Query<(&Parent, &CollidingEntities, &InteractRadius)>,
    player_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_query.into_inner();

    for (parent, colliding_entities, interact_radius) in query.iter() {
        // Check if any of the colliding entities is the player
        if colliding_entities.contains(&player_entity) {
            commands.trigger_targets(interact_radius.event.clone(), parent.get());
        }
    }
}
