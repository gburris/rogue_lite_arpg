use avian2d::prelude::Collider;
use bevy::prelude::*;
use bevy_bundled_observers::observers;

use super::{
    equipment::{on_healing_tome_cast, on_shield_block, EquipmentSlot},
    HealingTome, Holdable, Item, ItemType, Shield,
};
use crate::{
    combat::{
        mana::{ManaCost, ManaDrainRate},
        shield::components::ProjectileReflection,
    },
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::equipment::{EquipmentTransform, Equippable},
    prelude::*,
};

pub fn tome_of_healing(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Tome Of Healing"),
        Item::new(355, ItemType::Tome),
        Equippable::from(2.0, EquipmentSlot::Offhand),
        ManaCost(40.0),
        HealingTome {
            healing: (25.0, 50.0),
        },
        Sprite::from_image(sprites.tome_of_healing.clone()),
        observers![on_healing_tome_cast],
    )
}
pub fn magic_shield(
    sprites: &Res<SpriteAssets>,
    sprite_layouts: &Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Name::new("Magic Shield"),
        Item::new(355, ItemType::Tome),
        Equippable::from(0.5, EquipmentSlot::Offhand),
        ManaCost(5.0),
        ManaDrainRate(20.0),
        ProjectileReflection,
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        Holdable,
        Sprite {
            image: sprites.magic_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        observers![on_shield_block],
    )
}

pub fn knight_shield(
    sprites: &Res<SpriteAssets>,
    sprite_layouts: &Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Name::new("Knight Shield"),
        Item::new(355, ItemType::Tome),
        Equippable::from(0.5, EquipmentSlot::Offhand),
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        ManaDrainRate(25.0),
        ManaCost(25.0),
        Holdable,
        Sprite {
            image: sprites.knight_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        observers![on_shield_block],
    )
}
