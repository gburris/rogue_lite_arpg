use std::time::Instant;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::GameCollisionLayer, labels::states::AppState, map::events::CreateInstanceEvent,
    player::Player,
};

use super::{
    events::CleanupZone, helpers::map_layout::generate_map_layout, InstanceAssets, Mapper,
    WorldSpaceConfig,
};

/**
 * Portals represent any "warping device" in the game, currently spawning a new zone when entered
 */
#[derive(Component)]
#[require(
    RigidBody(|| RigidBody::Static),
    Collider(|| Collider::rectangle(32.0, 64.0)),
    CollidingEntities,
    Mapper,
    CollisionLayers(default_collision_layers),
)]
pub enum Portal {
    StartingPortal,
    WarpZone,
}

fn default_collision_layers() -> CollisionLayers {
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
                info!("Creating new instance");
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
    info!("Portal entered!");
    commands.trigger(CleanupZone);
    game_state.set(AppState::CreateInstance);
}

pub fn on_mapper_spawned(
    trigger: Trigger<OnAdd, Mapper>,
    mut commands: Commands,
    mut portal_query: Query<&mut Mapper, With<Portal>>,
    world_config: Res<WorldSpaceConfig>,
    instance_assets: Res<InstanceAssets>,
) {
    let start_time = Instant::now();

    let mut new_mapper = portal_query.get_mut(trigger.entity()).unwrap();
    new_mapper.map_layout = generate_map_layout(world_config.map_size, &instance_assets);
    commands.insert_resource(new_mapper.map_layout.clone());

    let duration = start_time.elapsed();
    warn!(
        "Finished setting the instance! Generation took: {:?}",
        duration
    );
}
