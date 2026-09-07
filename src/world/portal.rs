use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{prelude::*, world::MapRng};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_portal_collisions.in_set(InGameSystems::Collision),
    );

    app.add_observer(despawn_all::<CleanupZone, Portal>);
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
    YSort
)]
struct Portal;

pub fn portal(sprites: &SpriteAssets, position: Vec2) -> impl Bundle {
    (
        // Generate a unique instance layout for each portal
        Portal,
        Sprite::from_image(sprites.exit_door.clone()),
        Transform::from_translation(position.extend(ZLayer::OnGround.z())),
    )
}

fn handle_portal_collisions(
    mut commands: Commands,
    instance: Res<MapRecipeCatalog>,
    portal_query: Query<&CollidingEntities, With<Portal>>,
    player_collider: Single<Entity, With<PlayerInteractionRadius>>,
    mut game_state: ResMut<NextState<AppState>>,
    mut rng: ResMut<MapRng>,
) {
    for portal_colliding_entities in portal_query.iter() {
        for &colliding_entity in portal_colliding_entities.iter() {
            if colliding_entity == *player_collider {
                let (mut zone_rng, zone_seed) = rng.zone();
                info!("Generating portal zone with seed: {}", zone_seed);
                commands.insert_resource(
                    instance
                        .generate_blueprint(&mut zone_rng, zone_seed)
                        .unwrap(),
                );
                game_state.set(AppState::Transition);
            }
        }
    }
}
