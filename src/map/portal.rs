use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::GameCollisionLayer, labels::states::AppState, map::events::CreateInstanceEvent,
    player::Player,
};

use super::events::CleanupZone;

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
pub enum Portal {
    StartingPortal,
    WarpZone,
}

fn default_collision_layers() -> CollisionLayers {
    // Portals are sensors since we don't actually want collisions
    CollisionLayers::new(
        GameCollisionLayer::HighObstacle,
        GameCollisionLayer::HIGH_OBSTACLE_FILTERS,
    )
}

pub fn handle_portal_collisions(
    mut commands: Commands,
    portal_query: Query<&CollidingEntities, With<Portal>>,
    player_entity: Single<Entity, With<Player>>,
) {
    let player_entity = player_entity.into_inner();

    // If player is colliding with portal, we spawn a new instance
    for portal_colliding_entities in portal_query.iter() {
        for &colliding_entity in portal_colliding_entities.iter() {
            if colliding_entity == player_entity {
                commands.trigger(CreateInstanceEvent);
            }
        }
    }
}

pub fn on_portal_entered(
    _: Trigger<CreateInstanceEvent>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
) {
    commands.trigger(CleanupZone);
    game_state.set(AppState::CreateInstance);
}
