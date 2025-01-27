use super::{
    equipment::equipment_transform::DirectionTransforms, Consumable, ConsumableEffect,
    ConsumableType, Equippable, Helmet, ItemName, Shovel,
};
use crate::{
    combat::{
        attributes::mana::ManaCost,
        damage::components::CollisionDamage,
        projectile::components::ProjectileBundle,
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
        weapon::weapon::{on_weapon_fired, ProjectileWeapon, Weapon},
    },
    configuration::assets::SpriteAssets,
    items::{EquipmentSlot, HealthPotion, ItemId, Sword},
    player::movement::MovementDirection,
};
use bevy::prelude::*;

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
            DirectionTransforms::from(MovementDirection::Down).mainhand,
        ))
        .id()
}

pub fn spawn_sword(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Sword".to_string()),
            EquipmentSlot::Mainhand,
            Sword,
            Equippable::default(),
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.sword_equipment_sprite.clone()),
            DirectionTransforms::from(MovementDirection::Down).mainhand,
        ))
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
            DirectionTransforms::from(MovementDirection::Down).mainhand,
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
            DirectionTransforms::from(MovementDirection::Down).head,
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
            DirectionTransforms::from(MovementDirection::Down).mainhand,
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
            DirectionTransforms::from(MovementDirection::Down).mainhand,
        ))
        .id()
}
