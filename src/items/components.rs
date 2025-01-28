use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct ItemId(pub u32);

#[derive(Component, Clone, Debug)]
pub struct Consumable;

#[derive(Component, Clone, Debug, Default)]
pub struct ItemName(pub String);

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct Sword;

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct Shovel;

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct Helmet;

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct HealthPotion;

#[derive(Component)]
pub struct ConsumableEffect {
    pub effect_type: ConsumableType,
}

pub enum ConsumableType {
    Heal(f32), // Heal player for a specific amount
}
