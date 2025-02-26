use avian2d::prelude::Collider;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    animation::FacingDirection,
    combat::{
        attributes::mana::ManaCost,
        melee::components::{MeleeSwingType, MeleeWeapon},
        projectile::components::{Projectile, ProjectileBundle},
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
        weapon::weapon::ProjectileWeapon,
    },
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{
        equipment::{on_weapon_fired, on_weapon_melee, EquipmentTransform, Equippable},
        Item,
    },
};

use super::equipment::EquipmentSlot;

pub fn spawn_tome_of_healing(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    let offhand_transform: Transform = EquipmentTransform::get(FacingDirection::Down).offhand;

    commands
        .spawn((
            Name::new("Tome Of Healing"),
            Item::new(6),
            Equippable::from(2.0, EquipmentSlot::Offhand),
            ManaCost(40.0),
            Visibility::Hidden,
            Sprite::from_image(sprites.tome_of_healing.clone()),
            offhand_transform,
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_offhand(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
    offhand_name: &str,
) -> Entity {
    match offhand_name {
        "tome_of_healing" => spawn_tome_of_healing(commands, sprites),
        _ => unreachable!(), // Should never happen
    }
}
