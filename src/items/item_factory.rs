use avian2d::prelude::Collider;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use super::{
    equipment::{
        equipment_transform::DirectionTransforms,
        use_equipped::{on_weapon_fired, on_weapon_melee},
        Equippable,
    },
    Consumable, ConsumableEffect, ConsumableType, ItemName,
};
use crate::{
    animation::MovementDirection,
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
    items::{equipment::EquipmentSlot, HealthPotion, ItemId},
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
            MeleeWeapon {
                damage: 6.0,
                effects_list: EffectsList { effects: vec![] },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::stab(),
                attack_duration: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Equippable::default(),
            ItemId(3),
            Visibility::Visible,
            Sprite::from_image(sprites.sword.clone()),
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_axe(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Axe".to_string()),
            EquipmentSlot::Mainhand,
            MeleeWeapon {
                damage: 60.0,
                effects_list: EffectsList {
                    effects: vec![ApplyStatus {
                        status: StatusType::Frozen,
                        duration: 2.0,
                    }],
                },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::slash(),
                attack_duration: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Equippable::default(),
            ItemId(3),
            Visibility::Visible,
            Sprite::from_image(sprites.axe.clone()),
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
            Equippable::default(),
            ItemId(3),
            MeleeWeapon {
                damage: 6.0,
                effects_list: EffectsList { effects: vec![] },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::stab(),
                attack_duration: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Visibility::Hidden,
            Sprite::from_image(sprites.shovel_equipment_sprite.clone()),
            DirectionTransforms::get(MovementDirection::Down).mainhand,
        ))
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_helmet(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            ItemName("Helmet".to_string()),
            EquipmentSlot::Helmet,
            Equippable::default(),
            ItemId(3),
            Visibility::Hidden,
            Sprite::from_image(sprites.helmet_equipment_sprite.clone()),
            DirectionTransforms::get(MovementDirection::Down).head,
        ))
        .id()
}

pub fn spawn_fire_staff(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let fireball = ProjectileBundle {
        projectile: Projectile { damage: 6.0 },
        effects_list: EffectsList {
            effects: vec![ApplyStatus {
                status: StatusType::Burning(BurningStatus::default()),
                duration: 2.0,
            }],
        },
        sprite: Sprite::from_atlas_image(
            sprites.fire_ball.clone(),
            TextureAtlas {
                layout: texture_layouts.fireball_layout.clone(),
                index: 0,
            },
        ),
    };

    let weapon_transform: Transform = DirectionTransforms::get(MovementDirection::Down).mainhand;

    commands
        .spawn((
            ProjectileWeapon {
                projectile: fireball,
                projectile_speed: 700.0,
                spread: 0.0,
            },
            ManaCost(6.0),
            ItemName("Staff of Flames".to_string()),
            ItemId(6),
            EquipmentSlot::Mainhand,
            Equippable::default(),
            Visibility::Hidden,
            Sprite::from_image(sprites.fire_staff.clone()),
            weapon_transform,
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_ice_staff(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let icicle_projectile = ProjectileBundle {
        projectile: Projectile { damage: 25.0 }, // big damage
        effects_list: EffectsList {
            effects: vec![ApplyStatus {
                status: StatusType::Frozen,
                duration: 2.0,
            }],
        },
        sprite: Sprite::from_atlas_image(
            sprites.ice_bolt.clone(),
            TextureAtlas {
                layout: texture_layouts.ice_bolt_layout.clone(),
                index: 0,
            },
        ),
    };

    let weapon_transform: Transform = DirectionTransforms::get(MovementDirection::Down).mainhand;

    commands
        .spawn((
            ProjectileWeapon {
                projectile: icicle_projectile,
                projectile_speed: 500.0,
                spread: 0.0,
            },
            ItemName("Staff of Ice".to_string()),
            ItemId(6),
            ManaCost(20.0), // big mana cost
            EquipmentSlot::Mainhand,
            Equippable {
                use_rate: Timer::from_seconds(0.7, TimerMode::Once),
                ..default()
            },
            Visibility::Hidden,
            Sprite::from_image(sprites.ice_staff.clone()),
            weapon_transform,
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_random_mainhand_weapon(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let mut rng = thread_rng();
    let choice = rng.gen_range(0..4);

    match choice {
        0 => spawn_sword(commands, sprites),
        1 => spawn_axe(commands, sprites),
        2 => spawn_fire_staff(commands, sprites, texture_layouts),
        3 => spawn_ice_staff(commands, sprites, texture_layouts),
        _ => unreachable!(), // Should never happen
    }
}
