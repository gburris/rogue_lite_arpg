use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct ItemId(pub u32);

#[derive(Component, Clone, PartialEq, Debug)]
pub enum EquipmentSlot {
    Mainhand,
    Helmet,
}

#[derive(Component, Clone, Debug)]
pub struct Equippable {
    pub use_rate: Timer, // swing a sword, shoot a weapon, etc...
}

impl Default for Equippable {
    fn default() -> Self {
        Self {
            use_rate: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}

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
