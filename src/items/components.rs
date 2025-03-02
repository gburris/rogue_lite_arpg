use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, Sensor};
use bevy::prelude::*;

use crate::{configuration::GameCollisionLayer, despawn::components::LiveDuration};

/// This is the base component for all items in the game. If you have a new concept that will be
/// shared by all items, add it as a field here.
///
/// Ex.  All items can be dropped, so drop-related info can go here
#[derive(Component)]
pub struct Item {
    pub value: i32,
    pub item_type: ItemType,
    pub drop_glow_effect: f32,
    pub drop_rotation_timer: f32,
    pub drop_rate: f32,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            value: 10,
            item_type: ItemType::Melee,
            drop_glow_effect: 0.0,
            drop_rotation_timer: 0.0,
            drop_rate: 0.0,
        }
    }
}

impl Item {
    pub fn new(value: i32, item_type: ItemType) -> Self {
        Item {
            value,
            item_type,
            drop_rate: 1.2,
            ..default()
        }
    }
}

pub enum ItemType {
    Melee,
    Staff,
    Potion,
    Tome,
}

#[derive(Component, Clone, Debug)]
pub struct Consumable;

#[derive(Component, Clone, Debug)]
pub struct HealthPotion;

#[derive(Component, Clone, Debug)]
pub struct DropRate(pub f32);

#[derive(Component)]
pub struct ConsumableEffect {
    pub effect_type: ConsumableType,
}

pub enum ConsumableType {
    Heal(f32), // Heal player for a specific amount
}

#[derive(Event)]
pub struct ItemDropEvent;

#[derive(Component, Clone, Debug, Default)]
#[require(
    Visibility(|| Visibility::Visible),
    LiveDuration(|| LiveDuration::new(10.0))
)]
pub struct Lootable;

#[derive(Component)]
#[require(
    CollidingEntities,
    Sensor,
    Collider(|| Collider::circle(150.0)),
    CollisionLayers(|| CollisionLayers::new(
        GameCollisionLayer::Magnet,
        [GameCollisionLayer::Player]
    ))
)]
pub struct Magnet {
    pub strength: f32,
}

#[derive(Component)]
pub struct HealingTome {
    pub healing: (f32, f32),
}

#[derive(Component)]
#[require(
    Visibility(|| Visibility::Visible),
    LiveDuration(|| LiveDuration::new(1.0))
)]
pub struct HealingTomeSpellVisualEffect;
