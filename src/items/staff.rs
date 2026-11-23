use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};
use std::time::Duration;

use avian2d::prelude::SpatialQuery;
use bevy::prelude::*;
use bevy_tweening::lens::TransformRotateZLens;
use bevy_tweening::{Tween, TweenAnim};

use super::equipment::{DirectionalPoses, EquipmentPose};
use super::{Item, ItemKind};
use crate::prelude::*;

/// Farthest target distance at which an AI holder casts a staff.
pub const STAFF_REACH: f32 = 360.0;
/// Fraction of the standard fireball speed used by both fire mage staffs.
const MAGE_BOLT_SPEED_FACTOR: f32 = 0.7;
/// Seconds between the player's flame staff casts.
const FLAME_STAFF_COOLDOWN: f32 = 0.5;
/// Seconds between a fire mage's single-bolt volleys, leaving time to sidestep or step in between.
const MAGE_FLAME_STAFF_COOLDOWN: f32 = 1.1;
/// Seconds between a fire mage's three-bolt volleys: three times the bolts, fired half as often.
const MAGE_FAN_STAFF_COOLDOWN: f32 = 2.2;
/// Seconds between ice staff casts.
const ICE_STAFF_COOLDOWN: f32 = 1.0;
/// Seconds a flame staff takes to swing toward its target, and again to swing back.
const FLAME_STAFF_SWING_SECONDS: f32 = 0.1;
/// Seconds an ice staff takes to swing toward its target, and again to swing back.
const ICE_STAFF_SWING_SECONDS: f32 = 0.15;
/// Mana each flame staff cast costs.
const FLAME_STAFF_MANA_COST: f32 = 6.0;
/// Mana each single-bolt cast of a fire mage's staff costs.
const MAGE_FLAME_STAFF_MANA_COST: f32 = 10.0;
/// Mana each fan cast of a fire mage's staff costs: twice a single bolt, since it casts half as
/// often.
const MAGE_FAN_STAFF_MANA_COST: f32 = 2.0 * MAGE_FLAME_STAFF_MANA_COST;
/// Mana each ice staff cast costs.
const ICE_STAFF_MANA_COST: f32 = 15.0;

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    bsn! {
        @flame_staff_core(sprites, FLAME_STAFF_COOLDOWN, FLAME_STAFF_MANA_COST)
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
        @flame_staff_core(sprites, MAGE_FLAME_STAFF_COOLDOWN, MAGE_FLAME_STAFF_MANA_COST)
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
        @flame_staff_core(sprites, MAGE_FAN_STAFF_COOLDOWN, MAGE_FAN_STAFF_MANA_COST)
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

/// Shared item data, sprite and cast for every flame staff, casting every `seconds_between_casts`
/// for `mana_cost` mana.
fn flame_staff_core(
    sprites: &SpriteAssets,
    seconds_between_casts: f32,
    mana_cost: f32,
) -> impl Scene {
    bsn! {
        @staff(sprites.fire_staff.clone(), seconds_between_casts, FLAME_STAFF_SWING_SECONDS)
        ~Item::new(1340, ItemKind::Staff)
        ManaCost(mana_cost)
    }
}

pub fn ice_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    bsn! {
        @staff(sprites.ice_staff.clone(), ICE_STAFF_COOLDOWN, ICE_STAFF_SWING_SECONDS)
        Name::new("Staff of Ice")
        ~Item::new(2050, ItemKind::Staff)
        ManaCost(ICE_STAFF_MANA_COST)
        Projectiles [
            @icebolt(sprites, sprite_layouts, 0.0)
        ]
    }
}

/// What every staff shares: a mainhand item drawn with `image` that casts its projectiles every
/// `seconds_between_casts`. A cast winds up by swinging toward the target for `swing_seconds`,
/// fires, then swings back for as long.
fn staff(image: Handle<Image>, seconds_between_casts: f32, swing_seconds: f32) -> impl Scene {
    // The cooldown starts when the staff fires, at the end of its wind-up.
    let cooldown = (seconds_between_casts - swing_seconds).max(0.0);
    bsn! {
        Staff {
            source_offset: Vec2::new(0.0, 26.0),
            swing_seconds: swing_seconds,
        }
        ~Equippable::new(EquipmentSlot::Mainhand, &STAFF_EQUIPMENT_POSES)
        ~Cooldown::new(cooldown)
        ActionWindup { seconds: swing_seconds }
        ~{ActionUseStyle::Timed { seconds: swing_seconds }}
        ~{Reach(STAFF_REACH)}
        Attack
        Sprite { image }
        on(on_staff_cast_started)
        on(on_staff_took_effect)
        on(on_staff_finished)
    }
}

#[derive(Component, Default, Clone)]
struct Staff {
    /// X, y offset relative to center of staff sprite where projectiles should be spawned.
    source_offset: Vec2,
    swing_seconds: f32,
}

static STAFF_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::behind(10.0, 2.0, 0.0),
    down: EquipmentPose::above(-10.0, 0.0, 0.0),
    left: EquipmentPose::behind(-4.0, -7.0, 50.0),
    right: EquipmentPose::above(4.0, -8.0, -50.0),
};

/// Swings the staff toward its holder's facing and back, with a spark at its tip.
fn on_staff_cast_started(
    added: On<Add<ActionSourceInUse>>,
    mut commands: Commands,
    staff_query: Query<(&Staff, &Transform, &ActionSourceInUse)>,
    action_uses: Query<&ActionInUseOf>,
    holder_query: Query<&FacingDirection>,
) -> Result {
    let (staff, staff_transform, source_in_use) = staff_query.get(added.entity)?;
    let action_use = source_in_use.get();
    let &ActionInUseOf { holder } = action_uses.get(action_use)?;
    let Ok(facing_direction) = holder_query.get(holder) else {
        return Ok(());
    };

    let staff_rotation = staff_transform.rotation.to_euler(EulerRot::ZYX).0;
    let swing_angle = match facing_direction {
        FacingDirection::Up | FacingDirection::Right => -FRAC_PI_4,
        FacingDirection::Down | FacingDirection::Left => FRAC_PI_4,
    };
    let swing_duration = Duration::from_secs_f32(staff.swing_seconds);
    let swing_toward = Tween::new(
        EaseFunction::BackIn,
        swing_duration,
        TransformRotateZLens {
            start: staff_rotation,
            end: staff_rotation + swing_angle,
        },
    );
    let swing_back = Tween::new(
        EaseFunction::CubicOut,
        swing_duration,
        TransformRotateZLens {
            start: staff_rotation + swing_angle,
            end: staff_rotation,
        },
    );

    let tip = staff.source_offset.extend(ZLayer::AboveSprite.z());
    let staff_entity = added.entity;
    commands.spawn_scene(bsn! {
        @point_light(Color::WHITE, 10.0, 3.0, 1.0)
        Transform::from_translation(tip)
        ChildOf(staff_entity)
        ~{ActionUsePartOf { action_use }}
    });
    commands
        .entity(staff_entity)
        .insert(TweenAnim::new(swing_toward.then(swing_back)).with_destroy_on_completed(true));
    Ok(())
}

/// Fires the staff's projectiles from its tip at the end of its swing toward the target.
fn on_staff_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    staff_query: Query<(&Staff, &Transform)>,
    holder_query: Query<(&Transform, &Vision)>,
    spatial_query: SpatialQuery,
) -> Result {
    let (staff, staff_transform) = staff_query.get(took_effect.source)?;
    let Ok((holder_transform, holder_vision)) = holder_query.get(took_effect.holder) else {
        return Ok(());
    };

    let holder_relative_source_position = staff_transform
        .transform_point(staff.source_offset.extend(0.0))
        .xy();

    let target_angle: Vec2 =
        holder_vision.aim_direction * holder_vision.aim_distance - holder_relative_source_position;

    // Staff is child of holder so position is relative, need to add holder transform for
    // global position
    let holder_position = holder_transform.translation.truncate();
    let world_starting_position = pull_back_from_walls(
        &spatial_query,
        holder_position + Vec2::Y * CHARACTER_FEET_POS_OFFSET,
        holder_position + holder_relative_source_position,
    );

    commands.trigger(FireProjectiles {
        source: took_effect.source,
        holder: took_effect.holder,
        position: world_starting_position,
        aim_direction: target_angle,
    });
    Ok(())
}

fn on_staff_finished(finished: On<ActionFinished>, mut commands: Commands) {
    commands.entity(finished.source).try_remove::<TweenAnim>();
}
