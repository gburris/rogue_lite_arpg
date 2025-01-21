use super::{Consumable, ConsumableEffect, ConsumableType, Equipable, Helmet, ItemName, Shovel};
use crate::{
    combat::{
        damage::components::CollisionDamage,
        projectile::components::ProjectileBundle,
        spells::components::Spell,
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
        weapon::{projectile_weapon::ProjectileWeapon, weapon::Weapon},
    },
    configuration::assets::SpriteAssets,
    items::{EquipmentSlot, HealthPotion, ItemId, Sword},
};
use bevy::{math::Vec3, prelude::*, sprite::Sprite};

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
        ))
        .id()
}

pub fn spawn_sword(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Sword".to_string()),
            EquipmentSlot::Mainhand,
            Sword,
            Equipable,
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.sword_equipment_sripte.clone()),
            Transform::from_translation(Vec3::new(-65.0, -20.0, 0.1))
                .with_scale(Vec3::new(0.2, 0.3, 0.2))
                .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
        ))
        .id()
}

pub fn spawn_shovel(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Shovel".to_string()),
            EquipmentSlot::Mainhand,
            Shovel,
            Equipable,
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.shovel_equipment_sprite.clone()),
            Transform::from_translation(Vec3::new(-65.0, -20.0, 0.1))
                .with_scale(Vec3::new(0.2, 0.3, 0.2))
                .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
        ))
        .id()
}

pub fn spawn_helmet(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Helmet".to_string()),
            EquipmentSlot::Helmet,
            Helmet,
            Equipable,
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.helmet_equipment_sripte.clone()),
            Transform::from_translation(Vec3::new(-30.0, 40.0, 0.1))
                .with_scale(Vec3::new(0.2, 0.3, 0.2)),
        ))
        .id()
}

pub fn spawn_fire_staff(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ProjectileWeapon {
                projectile: ProjectileBundle {
                    spell: Spell::Fireball,
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
            Weapon::default(),
            ItemName("Staff of flames".to_string()),
            ItemId(6),
            EquipmentSlot::Mainhand,
            Equipable,
            Sprite::from_image(sprites.sword_equipment_sripte.clone()),
            Transform::from_translation(Vec3::new(-65.0, -20.0, 0.1))
                .with_scale(Vec3::new(0.2, 0.3, 0.2))
                .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
        ))
        .id()
}
