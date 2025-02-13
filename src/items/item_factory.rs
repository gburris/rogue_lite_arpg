use bevy::prelude::*;

use crate::{
    animation::FacingDirection,
    configuration::assets::SpriteAssets,
    items::{
        equipment::EquipmentTransform, Consumable, ConsumableEffect, ConsumableType, HealthPotion,
        Item,
    },
};

use super::{Charges, RechargeOnKill};

pub fn spawn_health_potion(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            HealthPotion,
            Name::new("Health Potion"),
            Item::new(1),
            ConsumableEffect {
                effect_type: ConsumableType::Heal(50.0), // Heals 50 HP
            },
            Consumable,
            Charges {
                current: 40, // Start with 40 charges
                max: 40,     // Maximum of 40 charges
            },
            RechargeOnKill,
            Visibility::Hidden,
            Sprite::from_image(sprites.health_potion.clone()),
            EquipmentTransform::get(FacingDirection::Down).mainhand,
        ))
        .id()
}
