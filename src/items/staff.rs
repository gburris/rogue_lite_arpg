use std::{
    f32::consts::{FRAC_PI_3, FRAC_PI_8},
    time::Duration,
};

use bevy::{prelude::*, ui_widgets::observe};
use bevy_lit::prelude::PointLight2d;
use bevy_tweening::{
    lens::{TransformRotateZLens, TransformRotationLens},
    *,
};

use crate::{
    items::{
        Item,
        equipment::{EquipmentType, Equippable},
        prelude::UseEquipment,
    },
    prelude::*,
};

use super::ItemType;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_casting_timer.in_set(InGameSystems::DespawnEntities),
            fire_projectile_on_casting_end.in_set(InGameSystems::Simulation),
            animate_staff_while_casting.in_set(InGameSystems::Vfx),
        ),
    );
}

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        Item::new(1340, ItemType::Staff),
        Equippable {
            equip_type: EquipmentType::Staff,
            use_rate: Timer::from_seconds(0.5, TimerMode::Once),
            ..default()
        },
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
            equip_type: EquipmentType::Staff,
            ..default()
        },
        Sprite::from_image(sprites.ice_staff.clone()),
        Projectiles::spawn_one(icebolt(sprites, sprite_layouts, 0.0)),
        observe(on_staff_fired),
    )
}

#[derive(Component)]
struct Casting {
    duration: Timer,
}

impl Casting {
    pub fn new(speed: f32) -> Self {
        Self {
            duration: Timer::from_seconds(speed, TimerMode::Once),
        }
    }
}

// "fired" implies this is a projectile weapon
fn on_staff_fired(
    staff: On<UseEquipment>,
    mut commands: Commands,
    staff_query: Query<&Transform, With<Projectiles>>,
) {
    let Ok(staff_transform) = staff_query.get(staff.entity) else {
        warn!("Unable to cast staff, no projectiles");
        return;
    };

    let staff_rotation = staff_transform.rotation.to_euler(EulerRot::ZYX).0;

    commands.entity(staff.entity).insert(Casting::new(0.3));

    commands.entity(staff.entity).insert(
        TweenAnim::new(
            // Create a tween that swings back and forth
            // Swing left
            Tween::new(
                EaseFunction::Linear,
                Duration::from_millis(100),
                TransformRotateZLens {
                    start: staff_rotation,
                    end: staff_rotation + FRAC_PI_8,
                },
            )
            // Swing right
            .then(Tween::new(
                EaseFunction::Linear,
                Duration::from_millis(100),
                TransformRotateZLens {
                    start: staff_rotation + FRAC_PI_8,
                    end: staff_rotation - FRAC_PI_8,
                },
            ))
            // Return
            .then(Tween::new(
                EaseFunction::Linear,
                Duration::from_millis(100),
                TransformRotateZLens {
                    start: staff_rotation - FRAC_PI_8,
                    end: staff_rotation,
                },
            )),
        )
        .with_destroy_on_completed(true),
    );
}

fn tick_casting_timer(casting_query: Query<&mut Casting>, time: Res<Time>) {
    for mut casting in casting_query {
        casting.duration.tick(time.delta());
    }
}

fn fire_projectile_on_casting_end(
    mut commands: Commands,
    staff_query: Query<(Entity, &Casting, &Projectiles, &ItemOf, &Sprite, &Transform)>,
    holder_query: Query<(&Transform, &Vision, Option<&TargetInfo>, Has<Enemy>)>,
    images: Res<Assets<Image>>,
) {
    for (staff_entity, casting, projectiles, item_of, sprite, staff_transform) in staff_query {
        if !casting.duration.is_finished() {
            continue;
        }

        let Ok((holder_transform, holder_vision, target_info, is_enemy)) =
            holder_query.get(item_of.0)
        else {
            warn!("Tried to fire staff with holder missing aim position or transform");
            continue;
        };

        let damage_source = if is_enemy {
            DamageSource::Enemy
        } else {
            DamageSource::Player
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

        // If no target ditance default to sane value away from holder
        let target_angle = (holder_vision.aim_direction * target_info.map_or(100., |t| t.distance))
            - staff_projectile_source_pos;

        // Staff is child of holder so position is relative, need to add holder transform for global position
        let starting_position =
            holder_transform.translation.truncate() + staff_projectile_source_pos;

        for projectile_entity in projectiles.iter() {
            commands.trigger(FireProjectile::from((
                projectile_entity,
                damage_source,
                starting_position,
                target_angle,
            )));
            commands.entity(staff_entity).remove::<Casting>();
        }
    }
}

fn animate_staff_while_casting(
    mut commands: Commands,
    staff_query: Query<(Entity, &Transform), With<Casting>>,
) {
    for (staff, transform) in staff_query {}
}
