use bevy::prelude::*;

use super::{
    equipment::{
        equipment_transform::DirectionTransforms,
        use_equipped::{on_weapon_fired, on_weapon_melee},
        Equippable,
    },
    Consumable, ConsumableEffect, ConsumableType, Helmet, ItemName, Shovel,
};
use crate::{
    combat::{
        attributes::mana::ManaCost,
        damage::components::CollisionDamage,
        melee::components::{MeleeHitbox, MeleeSwingPropertiesBundle, MeleeSwingType, MeleeWeapon},
        projectile::components::ProjectileBundle,
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
        weapon::weapon::{ProjectileWeapon, Weapon},
    },
    configuration::assets::SpriteAssets,
    items::{equipment::EquipmentSlot, HealthPotion, ItemId, Sword},
    player::movement::MovementDirection,
};

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
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .id()
}

pub fn spawn_sword(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Sword".to_string()),
            EquipmentSlot::Mainhand,
            Sword,
            Weapon,
            MeleeWeapon {
                melee_attack: MeleeSwingPropertiesBundle {
                    damage: CollisionDamage { damage: 6.0 },
                    effects_list: EffectsList { effects: vec![] },
                    hitbox: MeleeHitbox::default(),
                    sprite: Sprite::from_image(sprites.sword_equipment_sprite.clone()),
                    swing_type: MeleeSwingType::stab(),
                },
            },
            Equippable::default(),
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.sword_equipment_sprite.clone()),
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_shovel(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Shovel".to_string()),
            EquipmentSlot::Mainhand,
            Shovel,
            Equippable::default(),
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.shovel_equipment_sprite.clone()),
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .id()
}

pub fn spawn_helmet(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Helmet".to_string()),
            EquipmentSlot::Helmet,
            Helmet,
            Equippable::default(),
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.helmet_equipment_sprite.clone()),
            DirectionTransforms::get(MovementDirection::Down).head,
        ))
        .id()
}

pub fn spawn_fire_staff(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ProjectileWeapon {
                projectile: ProjectileBundle {
                    effects_list: EffectsList {
                        effects: vec![ApplyStatus {
                            status: StatusType::Burning(BurningStatus::default()),
                            duration: 2.0,
                        }],
                    },
                    sprite: Sprite::from(sprites.fire_bolt.clone()),
                    damage: CollisionDamage { damage: 6.0 },
                },
                spread: 0.0,
            },
            ManaCost(10.0),
            Weapon,
            ItemName("Staff of flames".to_string()),
            ItemId(6),
            EquipmentSlot::Mainhand,
            Equippable::default(),
            Visibility::Hidden,
            Sprite::from_image(sprites.staff_of_fire.clone()),
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_ice_staff(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ProjectileWeapon {
                projectile: ProjectileBundle {
                    effects_list: EffectsList {
                        effects: vec![ApplyStatus {
                            status: StatusType::Burning(BurningStatus::default()),
                            duration: 2.0,
                        }],
                    },
                    sprite: Sprite::from(sprites.fire_bolt.clone()),
                    damage: CollisionDamage { damage: 6.0 },
                },
                spread: 0.0,
            },
            Weapon,
            ItemName("Staff of flames".to_string()),
            ItemId(6),
            EquipmentSlot::Mainhand,
            Equippable::default(),
            Visibility::Hidden,
            Sprite::from_image(sprites.staff_of_fire.clone()),
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .id()
}
