use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::GameCollisionLayer, labels::states::AppState, map::components::SpawnZoneEvent,
    player::Player,
};

use super::components::MapLayout;

/**
 * Portals represent any "warping device" in the game, currently spawning a new zone when entered
 */
#[derive(Component)]
#[require(
    RigidBody(|| RigidBody::Static),
    Collider(|| Collider::rectangle(32.0, 64.0)),
    CollidingEntities,
    CollisionLayers(default_collision_layers),
)]
pub struct Portal {
    pub map_layout: MapLayout,
}

fn default_collision_layers() -> CollisionLayers {
    CollisionLayers::new(
        GameCollisionLayer::HighObstacle,
        GameCollisionLayer::HIGH_OBSTACLE_FILTERS,
    )
}

pub fn handle_portal_collisions(
    mut commands: Commands,
    portal_query: Query<(Entity, &CollidingEntities), With<Portal>>,
    player_entity: Single<Entity, With<Player>>,
) {
    let player_entity = player_entity.into_inner();
    for (entity, portal_colliding_entities) in portal_query.iter() {
        for &colliding_entity in portal_colliding_entities.iter() {
            if colliding_entity == player_entity {
                commands.trigger_targets(SpawnZoneEvent, entity);
            }
        }
    }
}

pub fn on_portal_entered(
    trigger: Trigger<SpawnZoneEvent>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    portal_query: Query<&Portal>,
) {
    if let Ok(portal) = portal_query.get(trigger.entity()) {
        commands.insert_resource(portal.map_layout.clone());
        game_state.set(AppState::SpawnZone);
    }
}
