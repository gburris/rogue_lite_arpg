use super::{
    Consumable, ConsumableEffect, ConsumableType, Equipable, EquipmentSprite, Helmet, ItemName,
};
use crate::{
    configuration::assets::SpriteAssets,
    items::{EquipmentSlot, HealthPotion, ItemId, Sword},
};
use bevy::{math::Vec3, prelude::*, sprite::Sprite};

pub fn spawn_health_potion(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ItemName("Health Potion".to_string()),
            ConsumableEffect {
                effect_type: ConsumableType::Heal(50.0), // Heals 50 HP
            },
            HealthPotion,
            ItemId(3),
            Consumable,
        ))
        .id()
}

pub fn spawn_sword(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Sword".to_string()),
            EquipmentSlot::Mainhand,
            Sword,
            Equipable,
            ItemId(3),
            Sprite::from_image(sprites.sword_equipment_sripte.clone()),
            EquipmentSprite {
                sprite: Sprite::from_image(sprites.sword_equipment_sripte.clone()),
                offset: Vec3::new(-65.0, -20.0, 0.1),
                scale: Vec3::new(0.2, 0.3, 0.2),
                rotation: Quat::from_rotation_z(90.0_f32.to_radians()),
            },
        ))
        .id()
}

pub fn spawn_helmet(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Helmet".to_string()),
            EquipmentSlot::Helmet,
            Helmet,
            Equipable,
            ItemId(3),
            Sprite::from_image(sprites.helmet_equipment_sripte.clone()),
            EquipmentSprite {
                sprite: Sprite::from_image(sprites.helmet_equipment_sripte.clone()),
                offset: Vec3::new(-30.0, 40.0, 0.1),
                scale: Vec3::new(0.2, 0.3, 0.2),
                rotation: Quat::default(),
            },
        ))
        .id()
}
