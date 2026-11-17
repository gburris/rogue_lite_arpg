use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};
use std::time::Duration;

use avian2d::prelude::SpatialQuery;
use bevy::prelude::*;
use bevy_tweening::lens::TransformRotateZLens;
use bevy_tweening::{Tween, TweenAnim};

use super::equipment::prelude::{DirectionalPoses, EquipmentPose};
use super::{Item, ItemKind};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        fire_staff_projectiles.in_set(SimulationSystems::Act),
    );
}

/// Farthest target distance at which an AI holder casts a staff.
pub const STAFF_REACH: f32 = 360.0;
/// Fraction of the standard fireball speed used by both fire mage staffs.
const MAGE_BOLT_SPEED_FACTOR: f32 = 0.7;
/// Seconds between the player's flame staff casts.
const FLAME_STAFF_COOLDOWN: f32 = 0.5;
/// Seconds between a fire mage's single-bolt volleys, leaving time to hop and sidestep in between.
const MAGE_FLAME_STAFF_COOLDOWN: f32 = 1.1;
/// Seconds between a fire mage's three-bolt volleys: three times the bolts, fired half as often.
const MAGE_FAN_STAFF_COOLDOWN: f32 = 2.2;

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    bsn! {
        @flame_staff_core(sprites, FLAME_STAFF_COOLDOWN)
        Name::new("Staff of Flames")
        Projectiles [
            @fireball(sprites, sprite_layouts, -FRAC_PI_8)
            --
            @fireball(sprites, sprite_layouts, 0.0)
            --
            @fireball(sprites, sprite_layouts, FRAC_PI_8)
        ]
    }
}

/// Fire-mage staff that fires a single bolt.
pub fn mage_fire_staff_single_bolt(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
) -> impl Scene {
    let speed = FIREBALL_SPEED * MAGE_BOLT_SPEED_FACTOR;
    bsn! {
        @flame_staff_core(sprites, MAGE_FLAME_STAFF_COOLDOWN)
        Name::new("Staff of Flames")
        Projectiles [
            @fireball_with_speed(sprites, sprite_layouts, 0.0, speed)
        ]
    }
}

/// Fire-mage staff that fires the full three-bolt fan.
pub fn mage_fire_staff_fan(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
) -> impl Scene {
    let speed = FIREBALL_SPEED * MAGE_BOLT_SPEED_FACTOR;
    bsn! {
        @flame_staff_core(sprites, MAGE_FAN_STAFF_COOLDOWN)
        Name::new("Staff of Wildfire")
        Projectiles [
            @fireball_with_speed(sprites, sprite_layouts, -FRAC_PI_8, speed)
            --
            @fireball_with_speed(sprites, sprite_layouts, 0.0, speed)
            --
            @fireball_with_speed(sprites, sprite_layouts, FRAC_PI_8, speed)
        ]
    }
}

/// Shared layout, item data, sprite, and cast observers for every flame staff.
fn flame_staff_core(sprites: &SpriteAssets, cooldown: f32) -> impl Scene {
    bsn! {
        @staff(sprites.fire_staff.clone(), cooldown)
        @staff_cast(0.1, 0.1)
        ~Item::new(1340, ItemKind::Staff)
        ManaCost(6.0)
    }
}

pub fn ice_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    bsn! {
        @staff(sprites.ice_staff.clone(), 1.0)
        Name::new("Staff of Ice")
        @staff_cast(0.15, 0.15)
        ~Item::new(2050, ItemKind::Staff)
        // big mana cost
        ManaCost(15.0)
        Projectiles [
            @icebolt(sprites, sprite_layouts, 0.0)
        ]
    }
}

/// What every staff shares: a mainhand item drawn with `image` that swings and casts its
/// projectiles every `cooldown` seconds.
fn staff(image: Handle<Image>, cooldown: f32) -> impl Scene {
    bsn! {
        Staff { source_offset: Vec2::new(0.0, 26.0) }
        ~Equippable::new(EquipmentSlot::Mainhand, &STAFF_EQUIPMENT_POSES)
        ~ActionCooldown::new(cooldown)
        ~{ActionReach(STAFF_REACH)}
        Attack
        Sprite { image }
        on(on_staff_took_effect)
        on(on_staff_finished)
    }
}

/// A staff's cast: it swings for `fire_time` seconds, fires, then swings back for `return_time`,
/// a timed use lasting both.
fn staff_cast(fire_time: f32, return_time: f32) -> impl Scene {
    bsn! {
        Staff { fire_time: fire_time, return_time: return_time }
        ~{ActionUseStyle::Timed { seconds: fire_time + return_time }}
    }
}

#[derive(Component, Default, Clone)]
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

/// Counts down a staff's swing toward its target, after which it fires.
#[derive(Component, Clone, Debug, Deref, DerefMut)]
struct StaffSwingProgress(Timer);

/// Begins the cast; projectiles fire later in `fire_staff_projectiles`.
fn on_staff_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    staff_query: Query<(&Staff, &Transform), With<Projectiles>>,
    holder_query: Query<&FacingDirection>,
) -> Result {
    let (staff, staff_transform) = staff_query.get(took_effect.source)?;
    let Ok(facing_direction) = holder_query.get(took_effect.holder) else {
        return Ok(());
    };

    let staff_rotation = staff_transform.rotation.to_euler(EulerRot::ZYX).0;

    let swing_angle = match facing_direction {
        FacingDirection::Up | FacingDirection::Right => -FRAC_PI_4,
        FacingDirection::Down | FacingDirection::Left => FRAC_PI_4,
    };

    commands.entity(took_effect.source).insert((
        StaffSwingProgress(Timer::from_seconds(staff.fire_time, TimerMode::Once)),
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
    ));

    // A spark at the tip while the cast winds up.
    let tip = staff.source_offset.extend(ZLayer::AboveSprite.z());
    let staff_entity = took_effect.source;
    commands.spawn_scene(bsn! {
        @point_light(Color::WHITE, 10.0, 3.0, 1.0)
        Transform::from_translation(tip)
        ChildOf(staff_entity)
    });
    Ok(())
}

/// Fires each staff's projectiles once its swing finishes. The swing back plays out the rest of
/// the timed use.
fn fire_staff_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut staff_query: Query<(Entity, &Staff, &mut StaffSwingProgress, &ItemOf, &Transform)>,
    holder_query: Query<(&Transform, &Vision)>,
    spatial_query: SpatialQuery,
) {
    for (staff_entity, staff, mut swing_progress, item_of, staff_transform) in &mut staff_query {
        if !swing_progress.tick(time.delta()).just_finished() {
            continue;
        }
        commands.entity(staff_entity).remove::<StaffSwingProgress>();

        let Ok((holder_transform, holder_vision)) = holder_query.get(item_of.holder) else {
            continue;
        };

        let rotated_source_offset =
            (staff_transform.rotation * staff.source_offset.extend(0.0)).truncate();
        let holder_relative_source_position =
            staff_transform.translation.truncate() + rotated_source_offset;

        let target_angle: Vec2 = holder_vision.aim_direction * holder_vision.aim_distance
            - holder_relative_source_position;

        // Staff is child of holder so position is relative, need to add holder transform for
        // global position
        let holder_position = holder_transform.translation.truncate();
        let world_starting_position = pull_back_from_walls(
            &spatial_query,
            holder_position + Vec2::Y * CHARACTER_FEET_POS_OFFSET,
            holder_position + holder_relative_source_position,
        );

        commands.trigger(FireProjectiles {
            source: staff_entity,
            holder: item_of.holder,
            position: world_starting_position,
            aim_direction: target_angle,
        });
    }
}

fn on_staff_finished(finished: On<ActionFinished>, mut commands: Commands) {
    commands
        .entity(finished.source)
        .remove::<(StaffSwingProgress, TweenAnim)>()
        .despawn_children();
}
