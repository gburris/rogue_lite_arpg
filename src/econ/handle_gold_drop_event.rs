use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, Sensor};
use bevy::prelude::*;
use rand::Rng;

use crate::{
    configuration::{assets::SpriteAssets, GameCollisionLayer},
    items::{Autoloot, Grounded, Magnet},
    labels::layer::ZLayer,
};

use super::components::{Currency, GoldDropEvent};

pub fn on_gold_drop_event(
    trigger: Trigger<GoldDropEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
) {
    let mut rng = rand::thread_rng();
    let mut entities_spawned = 0;
    let mut remaining_gold = trigger.amount;
    warn!("Spawning gold");
    // Calculate how many of each coin type to spawn
    while remaining_gold > 0 && entities_spawned < 10 {
        warn!("Spawning gold 1");
        let (sprite_path, value) = if remaining_gold >= 10000 {
            (sprites.gold_coin.clone(), 10000)
        } else if remaining_gold >= 1000 {
            (sprites.gold_coin.clone(), 1000)
        } else if remaining_gold >= 100 {
            (sprites.gold_coin.clone(), 100)
        } else if remaining_gold >= 10 {
            (sprites.gold_coin.clone(), 10)
        } else {
            (sprites.gold_coin.clone(), 1)
        };

        // Random position within radius
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(0.0..100.0);
        let offset = Vec2::new(angle.cos() * distance, angle.sin() * distance);

        let mut transform = trigger.drop_location;
        transform.translation.x += offset.x;
        transform.translation.y += offset.y;
        transform.translation.z = ZLayer::ItemOnGround.z();

        commands.spawn((
            Sprite::from_image(sprite_path),
            Autoloot,
            Magnet,
            transform,
            Currency { value: value },
            Grounded,
            Sensor,
            Collider::circle(100.0), //Magnet Radius
            CollisionLayers::new(GameCollisionLayer::Magnet, [GameCollisionLayer::Player]),
            CollidingEntities::default(),
        ));

        remaining_gold -= value;
        entities_spawned += 1;
    }
}
