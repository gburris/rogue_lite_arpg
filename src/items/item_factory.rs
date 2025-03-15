use bevy::prelude::*;

use crate::{
    animation::FacingDirection,
    configuration::assets::SpriteAssets,
    items::{
        equipment::EquipmentTransform, Consumable, ConsumableEffect, ConsumableType, HealthPotion,
        Item,
    },
};

use super::{lootable::on_lootable_item_interaction::on_lootable_item_interaction, ItemType};

pub fn on_item_added(trigger: Trigger<OnAdd, Item>, mut commands: Commands) {
    // We do this to avoid having to manually add this observer to every item we create
    commands
        .entity(trigger.entity())
        .observe(on_lootable_item_interaction);
}

pub fn spawn_health_potion(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            HealthPotion,
            Name::new("Health Potion"),
            Item::new(40, ItemType::Potion),
            ConsumableEffect {
                effect_type: ConsumableType::Heal(50.0), // Heals 50 HP
            },
            Consumable,
            Visibility::Hidden,
            Sprite::from_image(sprites.health_potion.clone()),
            EquipmentTransform::get(FacingDirection::Down).mainhand,
        ))
        .id()
}
