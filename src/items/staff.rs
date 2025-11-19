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
        observe(on_staff_fired),
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
        observe(on_staff_fired),
    )
}

// "fired" implies this is a projectile weapon
fn on_staff_fired(
    staff_fired: On<UseEquipment>,
    mut commands: Commands,
    staff_query: Query<(&Projectiles, &ItemOf, &Sprite, &Transform)>,
    holder_query: Query<(&Transform, &Vision, Option<&TargetInfo>)>,
    enemy_query: Query<Entity, With<Enemy>>,
    projectile_query: Query<(&Projectile, Option<&Effects>), With<Disabled>>,
    images: Res<Assets<Image>>,
) {
    let Ok((projectiles, item_of, sprite, staff_transform)) = staff_query.get(staff_fired.entity)
    else {
        warn!("Tried to fire staff that is not a projectile staff");
        return;
    };

    // TODO: Move staff "projectile source offset" computation to staff construction time!
    let staff_image = images
        .get(sprite.image.id())
        .expect("Staff should have image");

    // Subtract a little to try to get center of staff "eye" (source of projectile spawn)
    let staff_half_height = (staff_image.height() as f32 / 2.) - 6.;
    // Staff images start straight up (Vec3::Y) so multiply rotation quat by that direction to get direction as a unit vector
    let staff_projectile_source_offset =
        staff_half_height * (staff_transform.rotation * Vec3::Y).truncate();
    let staff_projectile_source_pos =
        staff_transform.translation.truncate() + staff_projectile_source_offset;

    let damage_source = if enemy_query.get(item_of.0).is_ok() {
        DamageSource::Enemy
    } else {
        DamageSource::Player
    };

    let Ok((holder_transform, holder_vision, target_info)) = holder_query.get(item_of.0) else {
        warn!("Tried to fire staff with holder missing aim position or transform");
        return;
    };

    // If no target ditance default to sane value away from holder
    let target_angle = (holder_vision.aim_direction * target_info.map_or(100., |t| t.distance))
        - staff_projectile_source_pos;

    for projectile_entity in projectiles.iter() {
        if let Ok((projectile, effects)) = projectile_query.get(projectile_entity) {
            trace!("Spawning projectile with effects: {:?}", effects);

            // Rotate the aim direction by the projectile’s angle offset
            let projectile_direction = target_angle
                .normalize()
                .rotate(Vec2::from_angle(projectile.angle_offset));

            // Staff is child of holder so position is relative, need to add holder transform for global position
            let starting_position =
                holder_transform.translation.truncate() + staff_projectile_source_pos;

            commands
                .entity(projectile_entity)
                .clone_and_spawn_with_opt_out(|builder| {
                    //builder.deny::<(Position, Rotation)>();
                    builder.linked_cloning(true);
                })
                .remove::<(ProjectileOf, Disabled)>()
                .insert((
                    Position(starting_position),
                    Rotation::radians(projectile_direction.to_angle()),
                    Transform {
                        translation: starting_position.extend(ZLayer::InAir.z()),
                        rotation: Quat::from_rotation_z(projectile_direction.to_angle()),
                        ..default()
                    },
                    LinearVelocity(projectile_direction * projectile.speed),
                    CollisionLayers::new(
                        GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                        LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
                    ),
                ));
        }
    }
}
