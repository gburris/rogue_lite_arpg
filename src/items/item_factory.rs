use bevy::prelude::*;

use super::{
    equipment::{equipment_transform::EquipmentTransform, Equippable},
    Consumable, ConsumableEffect, ConsumableType, Item,
};
use crate::{
    animation::FacingDirection,
    configuration::assets::SpriteAssets,
    items::{equipment::EquipmentSlot, HealthPotion},
};

pub fn spawn_health_potion(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            HealthPotion,
            Name::new("Health Potion"),
            Item::new(1),
            ConsumableEffect {
                effect_type: ConsumableType::Heal(50.0), // Heals 50 HP
            },
            Consumable,
            EquipmentTransform::get(FacingDirection::Down).mainhand,
        ))
        .id()
}
