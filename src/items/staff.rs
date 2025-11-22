use std::{
    f32::consts::{FRAC_PI_4, FRAC_PI_8},
    time::Duration,
};

use bevy::{prelude::*, ui_widgets::observe};
use bevy_lit::prelude::PointLight2d;
use bevy_tweening::{lens::TransformRotateZLens, *};

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
        ),
    );
}

#[derive(Component)]
struct Staff {
    /// x, y offset relative to center of staff sprite where projectiles should be spawned
    source_offset: Vec2,
}

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        Staff {
            source_offset: Vec2::new(0.0, 26.0),
        },
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
        Staff {
            source_offset: Vec2::new(0.0, 26.0),
        },
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
    staff_used: On<UseEquipment>,
    mut commands: Commands,
    staff_query: Query<(&Staff, &Transform, &ItemOf), With<Projectiles>>,
    holder_query: Query<&FacingDirection>,
) {
    let Ok((staff, staff_transform, item_of)) = staff_query.get(staff_used.entity) else {
        warn!("Unable to cast staff, no projectiles");
        return;
    };

    let Ok(facing_direction) = holder_query.get(item_of.0) else {
        warn!("Tried to fire staff with holder missing facing direction");
        return;
    };

    let staff_rotation = staff_transform.rotation.to_euler(EulerRot::ZYX).0;

    let swing_angle = match facing_direction {
        FacingDirection::Up => -FRAC_PI_4,
        FacingDirection::Down => FRAC_PI_4,
        FacingDirection::Left => FRAC_PI_4,
        FacingDirection::Right => -FRAC_PI_4,
    };

    commands
        .entity(staff_used.entity)
        .insert((
            Casting::new(0.1),
            TweenAnim::new(
                // Swing
                Tween::new(
                    EaseFunction::BackInOut,
                    Duration::from_millis(100),
                    TransformRotateZLens {
                        start: staff_rotation,
                        end: staff_rotation + swing_angle,
                    },
                )
                // Return
                .then(Tween::new(
                    EaseFunction::Linear,
                    Duration::from_millis(100),
                    TransformRotateZLens {
                        start: staff_rotation + swing_angle,
                        end: staff_rotation,
                    },
                )),
            )
            .with_destroy_on_completed(true),
        ))
        .with_child((
            Transform::from_translation(staff.source_offset.extend(ZLayer::AboveSprite.z())),
            PointLight2d {
                color: Color::WHITE,
                intensity: 10.0,
                outer_radius: 3.0,
                ..default()
            },
        ));
}

fn tick_casting_timer(casting_query: Query<&mut Casting>, time: Res<Time>) {
    for mut casting in casting_query {
        casting.duration.tick(time.delta());
    }
}

fn fire_projectile_on_casting_end(
    mut commands: Commands,
    staff_query: Query<(Entity, &Staff, &Casting, &Projectiles, &ItemOf, &Transform)>,
    holder_query: Query<(&Transform, &Vision, Option<&TargetInfo>, Has<Enemy>)>,
) {
    for (staff_entity, staff, casting, projectiles, item_of, staff_transform) in staff_query {
        if !casting.duration.is_finished() {
            continue;
        }

        commands
            .entity(staff_entity)
            .remove::<Casting>()
            .despawn_children();

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

        // Staff images start straight up (Vec3::Y) so multiply rotation quat by that direction to get direction as a unit vector
        let rotated_source_offset =
            (staff_transform.rotation * staff.source_offset.extend(0.0)).truncate();
        // staff.source_offset * (staff_transform.rotation * Vec3::Y).truncate();
        let staff_projectile_source_pos =
            staff_transform.translation.truncate() + rotated_source_offset;
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
        }
    }
}
