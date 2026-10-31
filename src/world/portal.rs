use avian2d::prelude::*;
use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, enter_portal.in_set(InGameSystems::Collision));
}

/// Portals represent any "warping device" in the game, currently spawning a new zone when entered.
#[derive(Component)]
#[require(
    RigidBody::Static,
    Collider::rectangle(TILE_SIZE as f32, 2.0 * TILE_SIZE as f32),
    CollidingEntities,
    CollisionLayers::new(
        GameCollisionLayer::Interaction,
        GameCollisionLayer::PlayerInteractionRadius
    ),
    YSort,
    DespawnOnExit::<AppState>(AppState::Playing)
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

fn enter_portal(
    portal_query: Query<&CollidingEntities, With<Portal>>,
    player_collider: Single<Entity, With<PlayerInteractionRadius>>,
    mut next_map: NextMap,
) -> Result {
    for portal_colliding_entities in &portal_query {
        if portal_colliding_entities.contains(&*player_collider) {
            return next_map.set(MapRequest::Weighted);
        }
    }

    Ok(())
}
