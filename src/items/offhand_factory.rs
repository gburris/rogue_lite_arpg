use bevy::prelude::*;

use super::equipment::on_healing_tome_cast;
use super::equipment::on_magic_shield_cast;
use super::equipment::on_shield_block;

use super::equipment::EquipmentSlot;
use super::HealingTome;
use super::Item;
use super::Shield;
use crate::animation::AnimationIndices;
use crate::animation::FacingDirection;
use crate::combat::attributes::mana::ManaCost;
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

//Two shields
//Melee Shield
//2 - When I cast, it should be in front of my player in the direction I am clicking
//3 - It should stay out for the length of it's cooldown
//4 - If a melee weapon collides with the shield, the active melee attack should despawn
//5 - The shield itself needs four sprites, back, left side, right side, front

//Magic shield
//2 - When I cast, it should create a spell reflection esque magic effect around me
//3 - This should have a pretty long cooldown, and reflect any spell that hits me for x seconds
//4 - Refelcted spells just fly in reverse and reset their live duration

fn spawn_magic_shield(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    let offhand_transform: Transform = EquipmentTransform::get(FacingDirection::Down).offhand;

    commands
        .spawn((
            Name::new("Magic Shield"),
            Item::new(6),
            Equippable::from(15.0, EquipmentSlot::Offhand),
            ManaCost(30.0),
            Visibility::Hidden,
            Sprite::from_image(sprites.magic_shield.clone()),
            offhand_transform,
        ))
        .observe(on_magic_shield_cast)
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
            Shield,
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
        "magic_shield" => spawn_magic_shield(commands, sprites),
        "knight_shield" => spawn_knight_shield(commands, sprites, layouts),
        _ => unreachable!(),
    }
}
