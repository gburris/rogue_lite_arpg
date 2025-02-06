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
