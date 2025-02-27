use bevy::prelude::*;

use super::equipment::on_healing_tome_cast;
use super::equipment::EquipmentSlot;
use super::HealingTome;
use super::Item;
use crate::animation::FacingDirection;
use crate::combat::attributes::mana::ManaCost;
use crate::configuration::assets::SpriteAssets;
use crate::items::equipment::EquipmentTransform;
use crate::items::equipment::Equippable;

pub fn spawn_tome_of_healing(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
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

pub fn spawn_offhand(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    offhand_name: &str,
) -> Entity {
    match offhand_name {
        "tome_of_healing" => spawn_tome_of_healing(commands, sprites),
        _ => unreachable!(), // Should never happen
    }
}
