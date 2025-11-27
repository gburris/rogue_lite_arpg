use std::{
    f32::consts::{FRAC_PI_4, FRAC_PI_8},
    sync::LazyLock,
    time::Duration,
};

use bevy::{platform::collections::HashMap, prelude::*, ui_widgets::observe};
use bevy_lit::prelude::PointLight2d;
use bevy_tweening::{Tween, TweenAnim, lens::TransformRotateZLens};

use crate::{
    equipment_transforms,
    items::{Item, equipment::Equippable, prelude::UseEquipment},
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

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        Staff {
            source_offset: Vec2::new(0.0, 26.0),
            fire_time: 0.1,
            return_time: 0.1,
        },
        Item::new(1340, ItemType::Staff),
        Equippable::new(EquipmentSlot::Mainhand, 0.5, &STAFF_EQUIPMENT_TRANSFORMS),
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
            fire_time: 0.15,
            return_time: 0.15,
        },
        Item::new(2050, ItemType::Staff),
        ManaCost(15.0), // big mana cost
        Equippable::new(EquipmentSlot::Mainhand, 1.0, &STAFF_EQUIPMENT_TRANSFORMS),
        Sprite::from_image(sprites.ice_staff.clone()),
        Projectiles::spawn_one(icebolt(sprites, sprite_layouts, 0.0)),
        observe(on_staff_fired),
    )
}

#[derive(Component)]
struct Staff {
    /// x, y offset relative to center of staff sprite where projectiles should be spawned
    source_offset: Vec2,
    fire_time: f32,
    return_time: f32,
}

use ZLayer::BehindSprite;
static STAFF_EQUIPMENT_TRANSFORMS: LazyLock<HashMap<FacingDirection, Transform>> =
    equipment_transforms!([
        (Up, ((10.0, 2.0), 0.0, BehindSprite)),
        (Down, ((-10.0, 0.0), 0.0)),
        (Left, ((-4.0, -7.0), 50.0, BehindSprite)),
        (Right, ((4.0, -8.0), -50.0)),
    ]);

#[derive(Component)]
struct Casting {
    fire_time: Timer,
    return_time: Timer,
}

impl Casting {
    pub fn new(fire_time: f32, return_time: f32) -> Self {
        let mut casting = Self {
            fire_time: Timer::from_seconds(fire_time, TimerMode::Once),
            return_time: Timer::from_seconds(return_time, TimerMode::Once),
        };

        casting.return_time.pause();

        casting
    }
}

// "fired" implies this is a projectile weapon
fn on_staff_fired(
    staff_used: On<UseEquipment>,
    mut commands: Commands,
    staff_query: Query<(&Staff, &Transform, &ItemOf), With<Projectiles>>,
    mut holder_query: Query<(&mut AttackState, &FacingDirection)>,
) {
    let Ok((staff, staff_transform, item_of)) = staff_query.get(staff_used.entity) else {
        warn!("Unable to cast staff, no projectiles");
        return;
    };

    let Ok((mut attack_state, facing_direction)) = holder_query.get_mut(item_of.0) else {
        warn!("Tried to fire staff with holder missing facing direction");
        return;
    };

    attack_state.is_attacking = true;

    let staff_rotation = staff_transform.rotation.to_euler(EulerRot::ZYX).0;

    let swing_angle = match facing_direction {
        FacingDirection::Up | FacingDirection::Right => -FRAC_PI_4,
        FacingDirection::Down | FacingDirection::Left => FRAC_PI_4,
    };

    commands
        .entity(staff_used.entity)
        .insert((
            Casting::new(staff.fire_time, staff.return_time),
            TweenAnim::new(
                // Swing
                Tween::new(
                    EaseFunction::BackIn,
                    Duration::from_secs_f32(staff.fire_time),
                    TransformRotateZLens {
                        start: staff_rotation,
                        end: staff_rotation + swing_angle,
                    },
                )
                // Return
                .then(Tween::new(
                    EaseFunction::CubicOut,
                    Duration::from_secs_f32(staff.return_time),
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
        casting.return_time.tick(time.delta());
        casting.fire_time.tick(time.delta());
    }
}

fn fire_projectile_on_casting_end(
    mut commands: Commands,
    staff_query: Query<(
        Entity,
        &Staff,
        &mut Casting,
        &Projectiles,
        &ItemOf,
        &Transform,
    )>,
    mut holder_query: Query<(
        &mut AttackState,
        &Transform,
        &Vision,
        Option<&TargetInfo>,
        Has<Enemy>,
    )>,
) {
    for (staff_entity, staff, mut casting, projectiles, item_of, staff_transform) in staff_query {
        if !casting.fire_time.is_finished() {
            continue;
        }

        let Ok((mut attack_state, holder_transform, holder_vision, target_info, is_enemy)) =
            holder_query.get_mut(item_of.0)
        else {
            warn!("Tried to fire staff with holder missing aim position or transform");
            continue;
        };

        // Time to fire, then we return
        if casting.return_time.is_paused() {
            casting.return_time.unpause();

            let rotated_source_offset =
                (staff_transform.rotation * staff.source_offset.extend(0.0)).truncate();
            let holder_relative_source_pos =
                staff_transform.translation.truncate() + rotated_source_offset;

            // If no target ditance default to sane value away from holder
            let target_angle: Vec2 = (holder_vision.aim_direction
                * target_info.map_or(100., |t| t.distance))
                - holder_relative_source_pos;

            // Staff is child of holder so position is relative, need to add holder transform for global position
            let world_starting_position =
                holder_transform.translation.truncate() + holder_relative_source_pos;

            for projectile_entity in projectiles.iter() {
                commands.trigger(FireProjectile::from((
                    projectile_entity,
                    DamageSource::from(is_enemy),
                    world_starting_position,
                    target_angle,
                )));
            }
        }

        // Return complete, cast attack is finished
        if casting.return_time.is_finished() {
            attack_state.is_attacking = false;

            commands
                .entity(staff_entity)
                .remove::<Casting>()
                .despawn_children();
        }
    }
}
