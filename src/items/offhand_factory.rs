use avian2d::prelude::Collider;
use bevy::prelude::*;

use super::equipment::on_healing_tome_cast;
use super::equipment::on_shield_block;

use super::equipment::EquipmentSlot;
use super::HealingTome;
use super::Holdable;
use super::Item;
use super::Shield;
use crate::animation::FacingDirection;
use crate::combat::attributes::mana::ManaCost;
use crate::combat::attributes::ManaDrainRate;
use crate::combat::shield::components::ProjectileReflection;
use crate::configuration::assets::SpriteAssets;
use crate::configuration::assets::SpriteSheetLayouts;
use crate::items::equipment::EquipmentTransform;
use crate::items::equipment::Equippable;

fn spawn_tome_of_healing(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    let offhand_transform: Transform = EquipmentTransform::get(FacingDirection::Down).offhand;

    commands
        .spawn((
            Name::new("Tome Of Healing"),
            Item::new(6),
            Equippable::from(2.0, EquipmentSlot::Offhand),
            ManaCost(40.0),
            HealingTome {
                healing: (25.0, 50.0),
            },
            Visibility::Hidden,
            Sprite::from_image(sprites.tome_of_healing.clone()),
            offhand_transform,
        ))
        .observe(on_healing_tome_cast)
        .id()
}

fn spawn_magic_shield(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let offhand_transform: Transform = EquipmentTransform::get(FacingDirection::Down).offhand;

    commands
        .spawn((
            Name::new("Magic Shield"),
            Item::new(6),
            Equippable::from(0.5, EquipmentSlot::Offhand),
            ManaCost(25.0),
            ManaDrainRate(25.0),
            ProjectileReflection,
            Shield {
                hitbox: Collider::rectangle(25.0, 25.0),
            },
            Holdable,
            Visibility::Hidden,
            Sprite {
                image: sprites.magic_shield.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layouts.shield_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            offhand_transform,
        ))
        .observe(on_shield_block)
        .id()
}

fn spawn_knight_shield(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let offhand_transform: Transform = EquipmentTransform::get(FacingDirection::Down).offhand;

    commands
        .spawn((
            Name::new("Knight Shield"),
            Item::new(6),
            Equippable::from(0.5, EquipmentSlot::Offhand),
            Shield {
                hitbox: Collider::rectangle(25.0, 25.0),
            },
            ManaDrainRate(25.0),
            ManaCost(25.0),
            Holdable,
            Visibility::Hidden,
            Sprite {
                image: sprites.knight_shield.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layouts.shield_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            offhand_transform,
        ))
        .observe(on_shield_block)
        .id()
}

pub fn spawn_offhand(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    layouts: &Res<SpriteSheetLayouts>,
    offhand_name: &str,
) -> Entity {
    match offhand_name {
        "tome_of_healing" => spawn_tome_of_healing(commands, sprites),
        "magic_shield" => spawn_magic_shield(commands, sprites, layouts),
        "knight_shield" => spawn_knight_shield(commands, sprites, layouts),
        _ => unreachable!(),
    }
}
