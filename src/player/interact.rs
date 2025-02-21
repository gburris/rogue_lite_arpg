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

impl InteractionZone {
    pub const OPEN_CHEST: Self = Self::Square { length: 50.0 };
    pub const NPC: Self = Self::Circle { radius: 70.0 };
    pub const ITEM_PICKUP: Self = Self::Circle { radius: 25.0 };
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
    interact_query: Query<(Entity, &Parent, &Transform, &CollidingEntities), With<InteractionZone>>,
    player_query: Single<(Entity, &Transform), With<Player>>,
) {
    let (player_entity, player_transform) = player_query.into_inner();
    let player_pos = player_transform.translation.truncate();

    // Go through all interaction zones colliding with something
    let closest_interaction: Option<(Entity, Entity, f32)> = interact_query
        .iter()
        // Filter for interaction zones colliding with player
        .filter(|(_, _, _, colliding)| colliding.contains(&player_entity))
        // Compute distance between player and each colliding zone
        .map(|(entity, parent, transform, _)| {
            let distance = (player_pos - transform.translation.truncate()).length();
            (entity, parent.get(), distance)
        })
        // Select colliding zone closest to player
        .min_by(|(_, _, dist_a), (_, _, dist_b)| dist_a.partial_cmp(dist_b).unwrap());

    if let Some((interaction_zone_entity, interactable_entity, _)) = closest_interaction {
        commands.trigger_targets(
            InteractionEvent {
                interaction_zone_entity,
            },
            interactable_entity,
        );
    }
}

/// This method acts as a constructor, adding a collider to the InteractionZone based the variant chosen
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
