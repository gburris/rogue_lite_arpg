use bevy::prelude::*;

use super::{
    equipment::{on_healing_tome_cast, EquipmentSlot},
    HealingTome, Item, ItemType,
};
use crate::{
    ai::state::FacingDirection,
    combat::mana::ManaCost,
    configuration::assets::SpriteAssets,
    items::equipment::{EquipmentTransform, Equippable},
};

fn spawn_tome_of_healing(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    let offhand_transform: Transform = EquipmentTransform::get(FacingDirection::Down).offhand;

    commands
        .spawn((
            Name::new("Tome Of Healing"),
            Item::new(355, ItemType::Tome),
            Equippable::from(2.0, EquipmentSlot::Offhand),
            ManaCost(40.0),
            HealingTome {
                healing: (25.0, 50.0),
            },
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
