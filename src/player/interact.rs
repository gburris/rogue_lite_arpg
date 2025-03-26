use avian2d::prelude::*;
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

/// Marker component for the sensor added to the player, which must collide with an InteractionZone for
/// a player interaction to be possible
#[derive(Component)]
pub struct PlayerInteractionRadius;

/// Component to be spawned as a child of any entity. When the player walks within "radius" and clicks "interact" (default: Spacebar)
/// this component will trigger the specified `[#InteractionEvent]` on the parent entity (ex. Open Chest, Talk to NPC, Item Pickup)
#[derive(Component)]
#[require(
    Sensor,
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, GameCollisionLayer::PlayerInteractionRadius))
)]
pub enum InteractionZone {
    Circle { radius: f32 },
    Square { length: f32 },
}

impl InteractionZone {
    pub const OPEN_CHEST: Self = Self::Square { length: 50.0 };
    pub const NPC: Self = Self::Circle { radius: 30.0 };
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
    interact_query: Query<(&Parent, &Transform), With<InteractionZone>>,
    player_query: Single<(&Transform, &CollidingEntities), With<PlayerInteractionRadius>>,
) {
    let (player_transform, interact_collisions) = player_query.into_inner();
    let player_pos = player_transform.translation.truncate();

    // Go through all interaction zones colliding with something
    let closest_interaction: Option<(Entity, Entity, f32)> = interact_collisions
        .iter()
        // Compute distance between player and each colliding zone
        .map(|&interact_entity| {
            let (parent, transform) = interact_query.get(interact_entity).expect("bruh");

            let distance = (player_pos - transform.translation.truncate()).length();
            (interact_entity, parent.get(), distance)
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

    commands.entity(trigger.entity()).insert((collider,));
}
