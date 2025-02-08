use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

#[derive(Component)]
pub struct Item {
    id: u32,
}

impl Item {
    pub fn new(id: u32) -> Self {
        Item { id }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Component, Clone, Debug)]
pub struct Consumable;

#[derive(Component, Clone, Debug)]
pub struct HealthPotion;

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

// Component to track glow effect animation
#[derive(Component)]
pub struct GroundedItemEffect {
    pub rotation: f32,
    pub glow_offset: f32,
}

impl Default for GroundedItemEffect {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            glow_offset: 0.0,
        }
    }
}
