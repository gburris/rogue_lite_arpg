use avian2d::prelude::*;
use bevy::prelude::*;

use crate::interval::Interval;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, collect_gold.in_set(InGameSystems::Collision))
        .add_systems(
            FixedUpdate,
            update_magnet_locations.in_set(MainSystems::InGame),
        )
        .add_observer(on_drop_gold);
}

#[derive(Component, Clone, Default)]
#[require(
    RigidBody,
    Collider::circle(10.0),
    CollisionLayers::new(
        [GameCollisionLayer::Grounded, GameCollisionLayer::Interaction],
        [
            GameCollisionLayer::PlayerInteractionRadius,
            GameCollisionLayer::HighObstacle,
            GameCollisionLayer::LowObstacle
        ]
    ),
    CollidingEntities,
    LockedAxes = LockedAxes::new().lock_rotation(),
    LinearDamping(2.0),
    TranslationExtrapolation,
    // Don't let gold move the player upon collision
    Dominance(-1),
    YSort,
    DespawnOnExit::<AppState>(AppState::Playing)
)]
struct Gold {
    pub value: u32,
}

const MAGNETIC_FORCE: f32 = 2_000_000.0;

#[derive(Component, Default, Clone)]
#[require(
    CollidingEntities,
    Sensor,
    Collider::circle(200.0),
    CollisionLayers::new(
        GameCollisionLayer::Interaction,
        [GameCollisionLayer::PlayerInteractionRadius]
    )
)]
struct Magnet;

fn collect_gold(
    mut commands: Commands,
    gold_query: Query<(Entity, &Gold, &CollidingEntities)>,
    mut player_purse: Single<&mut Purse, With<Player>>,
    player_collider_entity: Single<Entity, With<PlayerInteractionRadius>>,
) {
    let pe = player_collider_entity.into_inner();
    for (gold_entity, gold, colliding_entities) in &gold_query {
        if colliding_entities.contains(&pe) {
            player_purse.add(gold.value);
            commands.entity(gold_entity).despawn();
        }
    }
}

#[derive(Event)]
pub struct DropGold {
    pub location: Vec2,
    pub amount: u32,
}

const MAX_COINS_TO_SPAWN: i32 = 5;
/// How far from the drop point a dropped coin lands.
const GOLD_SCATTER_DISTANCE: Interval = Interval::new(20.0, 70.0);

fn on_drop_gold(gold_drop: On<DropGold>, mut commands: Commands, sprites: Res<SpriteAssets>) {
    let mut rng = rand::rng();
    let mut entities_spawned = 0;
    let mut remaining_gold = gold_drop.amount;
    // TODO: Give each visual representation of money quantity
    // It's own sprite. Like red, yellow and blue coins in Mario 64.
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

        let offset = Dir2::from_rng(&mut rng) * GOLD_SCATTER_DISTANCE.pick_random_value();

        commands.spawn_scene(gold(gold_image, value, gold_drop.location + offset));

        remaining_gold -= value;
        entities_spawned += 1;
    }
}

/// A coin worth `value`, pulled toward the player once they're close.
fn gold(gold_image: Handle<Image>, value: u32, location: Vec2) -> impl Scene {
    bsn! {
        Gold { value }
        Sprite { image: gold_image }
        Transform::from_translation({location.extend(ZLayer::OnGround.z())})
        Children [
            Magnet
        ]
    }
}

fn update_magnet_locations(
    magnet_query: Query<(&ChildOf, &GlobalTransform, &CollidingEntities), With<Magnet>>,
    player_query: Single<(Entity, &GlobalTransform), With<PlayerInteractionRadius>>,
    mut forces: Query<Forces>,
) {
    let (player_entity, player_transform) = player_query.into_inner();

    for (child_of, magnet_transform, colliding_entities) in &magnet_query {
        if colliding_entities.contains(&player_entity) {
            let magnet_to_player =
                player_transform.translation().xy() - magnet_transform.translation().xy();
            // Touching the player leaves no direction to pull along.
            let Some(direction) = magnet_to_player.try_normalize() else {
                continue;
            };

            // Measured in tiles so the distance isn't too large when cubed.
            let distance = magnet_to_player.length() / TILE_SIZE as f32;

            // https://en.wikipedia.org/wiki/Force_between_magnets#Magnetic_dipole_moment
            // Force gets stronger as magnet gets closer to ensure it closes in on target
            let magnetic_force = MAGNETIC_FORCE / distance.powi(3);

            trace!(
                "Magnetic force applied: {} from distance: {}",
                magnetic_force, distance
            );

            // Apply a new force each tick of fixed update, erasing previous force (persistence =
            // false)
            forces
                .get_mut(child_of.parent())
                .expect("Magnet parent missing or has no forces")
                .apply_force(direction * magnetic_force);
        }
    }
}

/// The gold a character carries: the player's savings, or what an enemy drops when defeated.
#[derive(Component, Clone, Copy, Default)]
pub struct Purse {
    pub amount: u32,
}

impl Purse {
    pub fn add(&mut self, amount: u32) {
        self.amount += amount;
    }
}
