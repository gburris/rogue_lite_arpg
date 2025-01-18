use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default)]
pub struct ItemId(u32);

#[derive(Component, Clone, Debug)]
pub enum EquipmentSlot {
    Mainhand,
    Helmet,
}

#[derive(Component, Clone, Debug, Default)]
pub struct ItemName(pub String);

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct Sword;

#[derive(Component, Clone, Debug)]
#[require(ItemName, ItemId)]
pub struct HealthPotion;

pub fn spawn_health_potion(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ItemName("Health Potion".to_string()),
            HealthPotion,
            ItemId(3),
        ))
        .id()
}

pub fn spawn_sword(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ItemName("Sword".to_string()),
            EquipmentSlot::Mainhand,
            Sword,
            ItemId(3),
        ))
        .id()
}
