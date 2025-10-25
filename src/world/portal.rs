use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::{GameCollisionLayer, YSort, ZLayer, assets::SpriteAssets},
    prelude::*,
};

use super::map::MapLayout;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_portal_collisions.in_set(InGameSystems::Collision),
    )
    .add_observer(on_portal_entered);
}

/// Portals represent any "warping device" in the game, currently spawning a new zone when entered
#[derive(Component)]
#[require(
    RigidBody::Static,
    Collider::rectangle(32.0, 64.0),
    CollidingEntities,
    CollisionLayers::new(
        GameCollisionLayer::Interaction,
        GameCollisionLayer::PlayerInteractionRadius
    ),
    YSort,
    DespawnOnExit::<AppState>(AppState::Playing),
)]
struct Portal {
    map_layout: MapLayout,
}

pub fn portal(instance: &InstanceAssets, sprites: &SpriteAssets, position: Vec2) -> impl Bundle {
    (
        // Generate a unique instance layout for each portal
        Portal {
            map_layout: instance.generate_map_layout().unwrap(),
        },
        Sprite::from_image(sprites.exit_door.clone()),
        Transform::from_translation(position.extend(ZLayer::OnGround.z())),
    )
}

fn handle_portal_collisions(
    mut commands: Commands,
    portal_query: Query<(Entity, &CollidingEntities), With<Portal>>,
    player_collider: Single<Entity, With<PlayerInteractionRadius>>,
) {
    for (entity, portal_colliding_entities) in portal_query.iter() {
        for &colliding_entity in portal_colliding_entities.iter() {
            if colliding_entity == *player_collider {
                commands.trigger(SpawnZone { entity });
            }
        }
    }
}

fn on_portal_entered(
    spawn_zone: On<SpawnZone>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    portal_query: Query<&Portal>,
) {
    if let Ok(portal) = portal_query.get(spawn_zone.entity) {
        commands.insert_resource(portal.map_layout.clone());
        game_state.set(AppState::SpawnZone);
    }
}
