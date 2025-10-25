use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    character::{Purse, player::interact::PlayerInteractionRadius},
    configuration::{GameCollisionLayer, YSort, assets::SpriteAssets},
    items::Magnet,
    prelude::{AppState, InGameSystems, Player},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_gold_collisions.in_set(InGameSystems::Collision),
    )
    .add_observer(on_gold_drop_event);
}

#[derive(Component)]
#[require(
    RigidBody,
    Collider::circle(10.0),
    CollisionLayers::new(
        [GameCollisionLayer::Grounded, GameCollisionLayer::Interaction],
        [GameCollisionLayer::PlayerInteractionRadius, GameCollisionLayer::HighObstacle, GameCollisionLayer::LowObstacle]
    ),
    CollidingEntities,
    LockedAxes = LockedAxes::new().lock_rotation(),
    LinearDamping(2.0),
    TranslationExtrapolation,
    // Don't let gold move the player upon collision
    Dominance(-1),
    YSort,
)]
struct Gold {
    pub value: u32,
}

fn handle_gold_collisions(
    mut commands: Commands,
    gold_query: Query<(Entity, &Gold, &CollidingEntities)>,
    mut player_purse: Single<&mut Purse, With<Player>>,
    player_collider_entity: Single<Entity, With<PlayerInteractionRadius>>,
) {
    let pe = player_collider_entity.into_inner();
    for (gold_entity, gold, colliding_entities) in gold_query.iter() {
        if colliding_entities.contains(&pe) {
            player_purse.add(gold.value);
            commands.entity(gold_entity).despawn();
        }
    }
}

#[derive(Event)]
pub struct GoldDrop {
    pub location: Vec2,
    pub amount: u32,
}

const MAX_COINS_TO_SPAWN: i32 = 5;

fn on_gold_drop_event(gold_drop: On<GoldDrop>, mut commands: Commands, sprites: Res<SpriteAssets>) {
    let mut rng = rand::rng();
    let mut entities_spawned = 0;
    let mut remaining_gold = gold_drop.amount;
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
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(20.0..70.0);
        let offset = Vec2::from_angle(angle) * distance;

        commands
            .spawn(gold(gold_image, value, gold_drop.location + offset))
            .with_child(Magnet);

        remaining_gold -= value;
        entities_spawned += 1;
    }
}

fn gold(gold_image: Handle<Image>, value: u32, location: Vec2) -> impl Bundle {
    (
        Gold { value },
        Sprite::from_image(gold_image),
        Transform::from_translation(location.extend(0.0)),
        DespawnOnExit(AppState::Playing),
    )
}
