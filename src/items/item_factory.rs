use bevy::prelude::*;

use super::{
    equipment::{equipment_transform::EquipmentTransform, Equippable},
    grounded::handle_item_to_ground::handle_item_ground_transition,
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
        .observe(handle_item_ground_transition)
        .id()
}

pub fn spawn_helmet(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            Equippable::new(EquipmentSlot::Helmet),
            Name::new("Helmet"),
            Item::new(2),
            Visibility::Hidden,
            Sprite::from_image(sprites.helmet_equipment_sprite.clone()),
            EquipmentTransform::get(FacingDirection::Down).head,
        ))
        .observe(handle_item_ground_transition)
        .id()
}
