use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct ItemId(pub u32);

#[derive(Component, Clone, PartialEq, Debug)]
pub enum EquipmentSlot {
    Mainhand,
    Helmet,
}

#[derive(Component, Clone, Debug)]
pub struct Equipable;

#[derive(Component, Clone, Debug)]
pub struct Consumable;

#[derive(Component, Clone, Debug, Default)]
pub struct ItemName(pub String);

#[derive(Component, Clone, Debug)]
pub struct EquipmentSprite {
    pub sprite: Sprite,
    pub offset: Vec3, // Offset from the player's position
    pub scale: Vec3,
    pub rotation: Quat, // Scale of the sprite
}

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct Sword;

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
