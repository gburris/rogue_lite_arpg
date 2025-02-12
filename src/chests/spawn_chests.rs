use avian2d::prelude::*;
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

use crate::{
    configuration::assets::SpriteAssets,
    map::systems::instance::spawn_instance_entities::ChestSpawnEvent,
};

#[derive(Component)]
#[require(
    Collider(|| Collider::rectangle(180.0, 50.0)),
    RigidBody(|| RigidBody::Static),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::LowObstacle, GameCollisionLayer::LOW_OBSTACLE_FILTERS))
)]
pub struct Chest;

#[derive(Component)]
#[require(
    CollidingEntities,
    Sensor,
    Collider(|| Collider::circle(100.0)),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, GameCollisionLayer::Player))
)]
pub struct ChestInteractionRadius;

pub fn spawn_chests(
    chest_spawn_trigger: Trigger<ChestSpawnEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
) {
    let chest_spawn_positions = chest_spawn_trigger.0.clone();
    for spawn_position in chest_spawn_positions {
        spawn_chest(&mut commands, &sprites, spawn_position);
    }
}

fn spawn_chest(commands: &mut Commands, sprites: &Res<'_, SpriteAssets>, spawn_position: Vec3) {
    commands
        .spawn((
            Chest,
            Sprite::from_image(sprites.closed_chest.clone()),
            Transform {
                translation: spawn_position,
                ..default()
            },
        ))
        .with_child(ChestInteractionRadius);
}
