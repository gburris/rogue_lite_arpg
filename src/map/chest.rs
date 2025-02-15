use avian2d::prelude::*;
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

use crate::configuration::assets::SpriteAssets;
use crate::econ::components::GoldDropEvent;
use crate::player::interact::{InteractionEvent, InteractionZone};

#[derive(Debug, Event)]
pub struct ChestSpawnEvent(pub Vec<Vec3>);

#[derive(Component)]
#[require(
    Collider(|| Collider::rectangle(180.0, 50.0)),
    RigidBody(|| RigidBody::Static),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::LowObstacle, GameCollisionLayer::LOW_OBSTACLE_FILTERS))
)]
pub struct Chest;

pub fn on_chest_spawn_event(
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
        .observe(on_interaction_open_chest)
        .with_child(InteractionZone { radius: 100.0 });
}

pub fn on_interaction_open_chest(
    open_chest_trigger: Trigger<InteractionEvent>,
    chest_transforms: Query<&Transform, With<Chest>>,
    sprites: Res<SpriteAssets>,
    mut commands: Commands,
) {
    let chest_entity = open_chest_trigger.entity();

    commands
        .entity(chest_entity)
        .insert(Sprite::from_image(sprites.open_chest.clone()));

    commands
        .entity(open_chest_trigger.interaction_zone_entity)
        .despawn();

    if let Ok(chest_transform) = chest_transforms.get(chest_entity) {
        commands.trigger(GoldDropEvent {
            amount: 999,
            drop_location: *chest_transform,
        });
    };
}
