use bevy::prelude::*;

use crate::{
    combat::{
        attributes::mana::ManaCost,
        damage::components::CollisionDamage,
        projectile::components::ProjectileBundle,
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
    },
    configuration::assets::SpriteAssets,
    items::{EquipmentSlot, Equippable, ItemId, ItemName},
    labels::layer::ZLayer,
};

use super::weapon::{on_weapon_fired, ProjectileWeapon, Weapon};

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
            Transform::from_translation(Vec3::new(-65.0, -20.0, 0.1))
                .with_scale(Vec3::new(0.2, 0.3, 0.2))
                .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
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
            Transform::from_translation(Vec3::new(-65.0, -20.0, ZLayer::Weapon.z()))
                .with_scale(Vec3::new(0.2, 0.3, 0.2))
                .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
        ))
        .id()
}
