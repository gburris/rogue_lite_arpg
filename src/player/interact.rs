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
pub enum InteractionZone {
    Circle { radius: f32 },
    Square { length: f32 },
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
    interact_query: Query<
        (Entity, &Parent, &Transform, &CollidingEntities),
        (With<InteractionZone>, With<CollidingEntities>),
    >,
    player_query: Single<(Entity, &Transform), With<Player>>,
) {
    let (player_entity, player_transform) = player_query.into_inner();

    // Go through all interaction zones colliding with player
    let closest_interaction: Option<(Entity, Entity, f32)> = interact_query
        .iter()
        .filter(|(_, _, _, colliding)| colliding.contains(&player_entity))
        .fold(None, |current, (new_entity, parent, next_transform, _)| {
            let new_distance = (player_transform.translation.truncate()
                - next_transform.translation.truncate())
            .abs()
            .length();

            if let Some(closest) = current {
                if closest.2 < new_distance {
                    Some((new_entity, parent.get(), new_distance))
                } else {
                    Some(closest)
                }
            } else {
                Some((new_entity, parent.get(), new_distance))
            }
        });

    if let Some((interaction_zone_entity, interactable_entity, _)) = closest_interaction {
        commands.trigger_targets(
            InteractionEvent {
                interaction_zone_entity,
            },
            interactable_entity,
        );
    }
}

pub fn on_interaction_zone_added(
    trigger: Trigger<OnAdd, InteractionZone>,
    mut commands: Commands,
    interact_query: Query<&InteractionZone>,
) {
    // We can unwrap since this is an OnAdd. Surely it exists right 0.o
    let interact = interact_query.get(trigger.entity()).unwrap();

    let collider = match interact {
        InteractionZone::Circle { radius } => Collider::circle(*radius),
        InteractionZone::Square { length } => Collider::rectangle(*length, *length),
    };

    commands.entity(trigger.entity()).insert(collider);
}
