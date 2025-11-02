use std::f32::consts::FRAC_PI_8;

use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*, ui_widgets::observe};

use crate::{
    items::{Item, equipment::Equippable, prelude::UseEquipment},
    prelude::*,
};

use super::ItemType;

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        Item::new(1340, ItemType::Staff),
        Equippable::default(),
        ManaCost(6.0),
        Sprite::from_image(sprites.fire_staff.clone()),
        related!(
            Projectiles [
                fireball(sprites, sprite_layouts, -FRAC_PI_8),
                fireball(sprites, sprite_layouts, 0.0),
                fireball(sprites, sprite_layouts, FRAC_PI_8)
            ]
        ),
        observe(on_weapon_fired),
    )
}

pub fn ice_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Ice"),
        Item::new(2050, ItemType::Staff),
        ManaCost(20.0), // big mana cost
        Equippable {
            use_rate: Timer::from_seconds(0.7, TimerMode::Once),
            ..default()
        },
        Sprite::from_image(sprites.ice_staff.clone()),
        Projectiles::spawn_one(icebolt(sprites, sprite_layouts, 0.0)),
        observe(on_weapon_fired),
    )
}

// "fired" implies this is a projectile weapon
fn on_weapon_fired(
    weapon_fired: On<UseEquipment>,
    mut commands: Commands,
    weapon_query: Query<(&Projectiles, &ItemOf)>,
    holder_query: Query<(&Transform, &Vision)>,
    enemy_query: Query<Entity, With<Enemy>>,
    projectile_query: Query<(&Projectile, Option<&Effects>), With<Disabled>>,
) {
    let Ok((projectiles, item_of)) = weapon_query.get(weapon_fired.entity) else {
        warn!("Tried to fire weapon that is not a projectile weapon");
        return;
    };

    let damage_source = if enemy_query.get(item_of.0).is_ok() {
        DamageSource::Enemy
    } else {
        DamageSource::Player
    };

    let Ok((holder_transform, holder_vision)) = holder_query.get(item_of.0) else {
        warn!("Tried to fire weapon with holder missing aim position or transform");
        return;
    };

    for projectile_entity in projectiles.iter() {
        if let Ok((projectile, effects)) = projectile_query.get(projectile_entity) {
            trace!("Spawning projectile with effects: {:?}", effects);

            // Rotate the aim direction by the projectile’s angle offset
            let rotated_direction = holder_vision
                .aim_direction
                .rotate(Vec2::from_angle(projectile.angle_offset));
            let starting_position = holder_transform.translation.truncate()
                + (projectile.forward_offset * rotated_direction);

            commands
                .entity(projectile_entity)
                .clone_and_spawn_with_opt_out(|builder| {
                    //builder.deny::<(Position, Rotation)>();
                    builder.linked_cloning(true);
                })
                .remove::<(ProjectileOf, Disabled)>()
                .insert((
                    Position(starting_position),
                    Rotation::radians(rotated_direction.to_angle()),
                    Transform {
                        translation: starting_position.extend(ZLayer::InAir.z()),
                        rotation: Quat::from_rotation_z(rotated_direction.to_angle()),
                        ..default()
                    },
                    LinearVelocity(rotated_direction * projectile.speed),
                    CollisionLayers::new(
                        GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                        LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
                    ),
                ));
        }
    }
}
