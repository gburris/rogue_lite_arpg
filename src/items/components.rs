use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

#[derive(Component)]
pub struct Item {
    id: u32,
    //For any data that is assoicated with all items, we should put it here
    pub drop_glow_effect: f32,
    pub drop_rotation_timer: f32,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            id: 0,
            drop_glow_effect: 0.0,
            drop_rotation_timer: 0.0,
        }
    }
}

impl Item {
    pub fn new(id: u32) -> Self {
        Item { id, ..default() }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
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
pub struct ItemToGroundEvent {
    pub origin_position: Vec3,
}

#[derive(Component, Clone, Debug)]
#[require(CollidingEntities)]
pub struct Grounded;
