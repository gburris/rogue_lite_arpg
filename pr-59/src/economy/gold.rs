use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    configuration::{assets::SpriteAssets, GameCollisionLayer, YSort},
    items::{inventory::Inventory, Magnet},
    player::{Player, PlayerCollider},
};

#[derive(Component)]
#[require(
    RigidBody,
    Collider(|| Collider::circle(10.0)),
    CollisionLayers(|| CollisionLayers::new(
        [GameCollisionLayer::Grounded],
        [GameCollisionLayer::PlayerCollider, GameCollisionLayer::HighObstacle, GameCollisionLayer::LowObstacle]
    )),
    CollidingEntities,
    LockedAxes(|| LockedAxes::new().lock_rotation()),
    LinearDamping(|| LinearDamping(2.0)),
    TranslationExtrapolation,
    // Don't let gold move the player upon collision
    Dominance(|| Dominance(-1)),
    YSort,
)]
pub struct Gold {
    pub value: u32,
}

pub fn handle_gold_collisions(
    mut commands: Commands,
    gold_query: Query<(Entity, &Gold, &CollidingEntities)>,
    mut player_inventory: Single<&mut Inventory, With<Player>>,
    player_collider: Query<Entity, With<PlayerCollider>>,
) {
    let Ok(player_collider_entity) = player_collider.get_single() else {
        return;
    };

    for (gold_entity, gold, colliding_entities) in gold_query.iter() {
        if colliding_entities.contains(&player_collider_entity) {
            player_inventory.add_coins(gold.value);
            commands.entity(gold_entity).despawn_recursive();
        }
    }
}

#[derive(Event)]
pub struct GoldDropEvent {
    pub drop_location: Vec2,
    pub amount: u32,
}

const MAX_COINS_TO_SPAWN: i32 = 5;

pub fn on_gold_drop_event(
    trigger: Trigger<GoldDropEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
) {
    let mut rng = rand::thread_rng();
    let mut entities_spawned = 0;
    let mut remaining_gold = trigger.amount;
    //TODO: Give each visual representation of money quantity
    //It's own sprite. Like red, yellow and blue coins in Mario 64.
    while remaining_gold > 0 && entities_spawned < MAX_COINS_TO_SPAWN {
        let (gold_image, mut value) = match remaining_gold {
            n if n >= 10000 => (sprites.gold_coin.clone(), 10000),
            n if n >= 1000 => (sprites.gold_coin.clone(), 1000),
            n if n >= 100 => (sprites.gold_coin.clone(), 100),
            n if n >= 10 => (sprites.gold_coin.clone(), 10),
            _ => (sprites.gold_coin.clone(), 1),
        };

        // If we are spawning the last gold entity, include remaining gold
        if entities_spawned == MAX_COINS_TO_SPAWN - 1 {
            value = remaining_gold;
        }

        // Random position within radius
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(20.0..70.0);
        let offset = Vec2::from_angle(angle) * distance;

        commands
            .spawn((
                Gold { value },
                Sprite::from_image(gold_image),
                Transform::from_translation((trigger.drop_location + offset).extend(0.0)),
            ))
            .with_child(Magnet);

        remaining_gold -= value;
        entities_spawned += 1;
    }
}
