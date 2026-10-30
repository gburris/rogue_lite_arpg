use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};
use std::time::Duration;

use bevy::prelude::*;
use bevy::ui_widgets::observe;
use bevy_lit::prelude::PointLight2d;
use bevy_tweening::lens::TransformRotateZLens;
use bevy_tweening::{Tween, TweenAnim};

use super::{Item, ItemType};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_casting_timer.in_set(InGameSystems::DespawnEntities),
            fire_projectile_on_casting_end.in_set(SimulationSystems::Logic),
        ),
    );
}

/// Farthest target distance at which an AI holder casts a staff.
pub const STAFF_REACH: f32 = 360.0;
/// Fraction of the standard fireball speed used by the fire mage's single-bolt staff.
const MAGE_SINGLE_BOLT_SPEED_FACTOR: f32 = 0.7;
/// Fraction of the standard fireball speed used by the fire mage's three-bolt staff.
const MAGE_FAN_SPEED_FACTOR: f32 = 0.2;

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        flame_staff_core(sprites),
        related!(
            Projectiles [
                fireball(sprites, sprite_layouts, -FRAC_PI_8),
                fireball(sprites, sprite_layouts, 0.0),
                fireball(sprites, sprite_layouts, FRAC_PI_8)
            ]
        ),
    )
}

/// Fire-mage staff that fires a single bolt.
pub fn mage_fire_staff_single_bolt(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        flame_staff_core(sprites),
        Projectiles::spawn_one(fireball_with_speed(
            sprites,
            sprite_layouts,
            0.0,
            FIREBALL_SPEED * MAGE_SINGLE_BOLT_SPEED_FACTOR,
        )),
    )
}

/// Fire-mage staff that fires the full three-bolt fan.
pub fn mage_fire_staff_fan(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
) -> impl Bundle {
    let fan_speed = FIREBALL_SPEED * MAGE_FAN_SPEED_FACTOR;

    (
        Name::new("Staff of Wildfire"),
        flame_staff_core(sprites),
        related!(
            Projectiles [
                fireball_with_speed(sprites, sprite_layouts, -FRAC_PI_8, fan_speed),
                fireball_with_speed(sprites, sprite_layouts, 0.0, fan_speed),
                fireball_with_speed(sprites, sprite_layouts, FRAC_PI_8, fan_speed)
            ]
        ),
    )
}

/// Shared layout, item data, sprite, and cast observers for every flame staff.
fn flame_staff_core(sprites: &SpriteAssets) -> impl Bundle {
    (
        Staff {
            source_offset: Vec2::new(0.0, 26.0),
            fire_time: 0.1,
            return_time: 0.1,
        },
        Item::new(1340, ItemType::Staff),
        Equippable::new(EquipmentSlot::Mainhand, 0.5, &STAFF_EQUIPMENT_POSES)
            .with_reach(STAFF_REACH),
        ManaCost(6.0),
        Sprite::from_image(sprites.fire_staff.clone()),
        observe(on_start_staff),
        observe(on_stop_staff),
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
        Equippable::new(EquipmentSlot::Mainhand, 1.0, &STAFF_EQUIPMENT_POSES)
            .with_reach(STAFF_REACH),
        Sprite::from_image(sprites.ice_staff.clone()),
        Projectiles::spawn_one(icebolt(sprites, sprite_layouts, 0.0)),
        observe(on_start_staff),
        observe(on_stop_staff),
    )
}

#[derive(Component)]
struct Staff {
    /// X, y offset relative to center of staff sprite where projectiles should be spawned.
    source_offset: Vec2,
    fire_time: f32,
    return_time: f32,
}

static STAFF_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::behind(10.0, 2.0, 0.0),
    down: EquipmentPose::above(-10.0, 0.0, 0.0),
    left: EquipmentPose::behind(-4.0, -7.0, 50.0),
    right: EquipmentPose::above(4.0, -8.0, -50.0),
};

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

/// Start observer: begins the cast (projectiles fire later in `fire_projectile_on_casting_end`).
fn on_start_staff(
    staff_used: On<StartUsingEquipment>,
    mut commands: Commands,
    staff_query: Query<(&Staff, &Transform), With<Projectiles>>,
    holder_query: Query<&FacingDirection>,
) {
    let Ok((staff, staff_transform)) = staff_query.get(staff_used.item) else {
        warn!("Unable to cast staff, no projectiles");
        return;
    };

    let Ok(facing_direction) = holder_query.get(staff_used.holder) else {
        warn!("Tried to fire staff with holder missing facing direction");
        return;
    };

    let staff_rotation = staff_transform.rotation.to_euler(EulerRot::ZYX).0;

    let swing_angle = match facing_direction {
        FacingDirection::Up | FacingDirection::Right => -FRAC_PI_4,
        FacingDirection::Down | FacingDirection::Left => FRAC_PI_4,
    };

    commands
        .entity(staff_used.item)
        .insert((
            EquipmentInUse,
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
    holder_query: Query<(&Transform, &Vision, Option<&TargetInfo>, &CombatSide)>,
) {
    for (staff_entity, staff, mut casting, projectiles, item_of, staff_transform) in staff_query {
        if !casting.fire_time.is_finished() {
            continue;
        }

        let Ok((holder_transform, holder_vision, target_info, side)) =
            holder_query.get(item_of.holder)
        else {
            warn!("Tried to fire staff with holder missing aim position or transform");
            continue;
        };

        // Time to fire, then we return
        if casting.return_time.is_paused() {
            casting.return_time.unpause();

            let rotated_source_offset =
                (staff_transform.rotation * staff.source_offset.extend(0.0)).truncate();
            let holder_relative_source_position =
                staff_transform.translation.truncate() + rotated_source_offset;

            // If no target ditance default to sane value away from holder
            let target_angle: Vec2 = (holder_vision.aim_direction
                * target_info.map_or(100., |t| t.distance))
                - holder_relative_source_position;

            // Staff is child of holder so position is relative, need to add holder transform for
            // global position
            let world_starting_position =
                holder_transform.translation.truncate() + holder_relative_source_position;

            for projectile_entity in projectiles.iter() {
                commands.trigger(FireProjectile {
                    item: projectile_entity,
                    side: *side,
                    holder: Some(item_of.holder),
                    position: world_starting_position,
                    aim_direction: target_angle,
                });
            }
        }

        // Return complete, cast attack is finished; the stop observer owns the removal.
        if casting.return_time.is_finished() {
            commands.trigger(StopUsingEquipment {
                item: staff_entity,
                holder: item_of.holder,
            });
        }
    }
}

fn on_stop_staff(stop: On<StopUsingEquipment>, mut commands: Commands) {
    commands
        .entity(stop.item)
        .remove::<(EquipmentInUse, Casting, TweenAnim)>()
        .despawn_children();
}
