use super::{Consumable, ConsumableEffect, ConsumableType, Equipable, ItemName};
use crate::items::{EquipmentSlot, HealthPotion, ItemId, Sword};
use bevy::prelude::{Commands, Entity};

pub fn spawn_health_potion(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ItemName("Health Potion".to_string()),
            ConsumableEffect {
                effect_type: ConsumableType::Heal(50.0), // Heals 50 HP
            },
            ItemId(3),
            Consumable,
        ))
        .id()
}

pub fn spawn_sword(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ItemName("Sword".to_string()),
            EquipmentSlot::Mainhand,
            Sword,
            Equipable,
            ItemId(3),
        ))
        .id()
}
