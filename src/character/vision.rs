use std::f32::consts::FRAC_PI_4;

use avian2d::prelude::{RayCaster, RayHits};
use bevy::{
    color::palettes::css::{GREEN, YELLOW},
    prelude::*,
};

use crate::{combat::damage::DamageDealtEvent, prelude::*, utility::schedule_component_removal};

/// Represents the current direction an entity is aiming toward (e.g., cursor for player, target for AI).
/// This is decoupled from movement-facing direction.
#[derive(Component, Default)]
pub struct Vision {
    pub aim_direction: Vec2,
}

/// Defines an entity's vision properties, such as how far it can see and how wide its vision cone is.
/// Immutable component.
#[derive(Component)]
#[component(immutable)]
#[require(Vision, TargetInfo)]
pub struct VisionCapabilities {
    /// Half-angle of the entity's field of view, in **radians**.
    ///
    /// For example:
    /// - `PI / 4.0` → 45° field on either side (90° total FOV)
    /// - `PI / 2.0` → 90° field on either side (180° total FOV)
    pub vision_cone_radius: f32,
}

impl Default for VisionCapabilities {
    fn default() -> Self {
        Self {
            vision_cone_radius: FRAC_PI_4,
        }
    }
}

/// Stores calculated perception data for a given potential or current target.
#[derive(Component, Default)]
pub struct TargetInfo {
    /// Distance to the observed entity.
    pub distance: f32,
    /// Direction vector pointing to the observed entity.
    pub direction: Vec2,
    /// Whether the observed entity is within an unobstructed line of sight (based on RayCaster).
    pub line_of_sight: bool,
    /// Whether the observed entity is within the entity’s vision cone angle.
    pub in_vision_cone: bool,
    // TODO: Add `last_known_position` for memory/prediction systems.
}

/// Marks that the entity is currently targeting another entity.
/// Added when both line of sight and vision cone are satisfied.
#[derive(Component)]
#[relationship(relationship_target = TargetedBy)]
pub struct Targeting(pub Entity);

/// Inverse of `Targeting` — tracked by the targeted entity.
#[derive(Component)]
#[relationship_target(relationship = Targeting)]
pub struct TargetedBy(Vec<Entity>);

/// Tracks which entity the NPC is currently watching or trying to detect.
#[derive(Component)]
#[relationship(relationship_target = WatchedBy)]
pub struct Watching(pub Entity);

/// Inverse of `Watching` — attached to the entity being watched.
#[derive(Component)]
#[relationship_target(relationship = Watching)]
pub struct WatchedBy(Vec<Entity>);

/// Indicates a temporary target lock.
/// Automatically removed after a certain duration via `Lifespan`.
#[derive(Component)]
pub struct TargetLock;

// ---------------------
// VISION + PERCEPTION
// ---------------------

/// Updates the `Vision` component's direction for each entity:
/// - If the entity is actively targeting something, aim at it.
/// - Otherwise, aim in the direction it is facing.
pub fn update_aim_position(
    mut character_query: Query<
        (&mut Vision, &TargetInfo, Has<Targeting>, &FacingDirection),
        Without<Player>,
    >,
) {
    character_query
        .par_iter_mut()
        .for_each(|(mut vision, target_info, has_target, facing_dir)| {
            vision.aim_direction = if has_target {
                target_info.direction
            } else {
                facing_dir.to_vec2()
            };
        });
}

/// Updates the direction and distance of the watched (or targeted) entity,
/// and points the `RayCaster` in that direction.
pub fn update_target_info(
    mut npc_query: Query<(
        &mut TargetInfo,
        &mut RayCaster,
        &Transform,
        &Watching,
        Option<&Targeting>,
    )>,
    target_query: Query<&Transform>,
) {
    npc_query.par_iter_mut().for_each(
        |(mut target_info, mut ray_caster, transform, watching, targeting)| {
            // Track distance and direction to target if there is one, otherwise track watching
            let target_entity = targeting.map(|t| t.0).unwrap_or(watching.0);

            if let Ok(target_transform) = target_query.get(target_entity) {
                let target_direction = (target_transform.translation.xy()
                    - transform.translation.xy())
                .normalize_or_zero();

                let target_distance = target_transform
                    .translation
                    .xy()
                    .distance(transform.translation.xy());

                target_info.direction = target_direction;
                target_info.distance = target_distance;

                ray_caster.direction = Dir2::new(target_direction).unwrap_or(Dir2::X);
            }
        },
    );
}

/// System that updates `TargetInfo.line_of_sight` and `TargetInfo.in_vision_cone`
/// based on whether an NPC sees its watched entity (via raycast + field of view).
///
/// This system:
/// - Checks if the watched entity (or any of its children) was hit by the NPC's vision ray
/// - Checks if the direction to the watched entity is within the vision cone
///
/// Requirements:
/// - The NPC must have a `Watching` component referencing the target
/// - The target must have a `WatchedBy` component and optionally `Children` (e.g. for colliders)
pub fn is_target_in_sight(
    mut npc_query: Query<(
        &mut TargetInfo,
        &RayHits,
        &Vision,
        &VisionCapabilities,
        &Watching,
        Option<&Targeting>,
    )>,
    target_query: Query<Option<&Children>, Or<(With<WatchedBy>, With<TargetedBy>)>>,
) {
    npc_query.par_iter_mut().for_each(
        |(mut target_info, ray_hits, vision, vision_capabilities, watching, targeting)| {
            target_info.line_of_sight = false;

            // Check if target is in vision cone angle
            let vision_cone_dot = vision_capabilities.vision_cone_radius.cos();
            target_info.in_vision_cone =
                target_info.direction.dot(vision.aim_direction) > vision_cone_dot;

            let target_entity = targeting.map(|t| t.0).unwrap_or(watching.0);

            for hit in ray_hits.iter() {
                // Check direct match
                if hit.entity == target_entity {
                    target_info.line_of_sight = true;
                    break;
                }

                // Check if hit entity is a child of the watched entity
                if let Ok(Some(children)) = target_query.get(target_entity) {
                    if children.contains(&hit.entity) {
                        target_info.line_of_sight = true;
                        break;
                    }
                }
            }
        },
    );
}

/// Draws debug gizmos for AI vision direction and cone angles.
pub fn debug_vision(mut gizmos: Gizmos, query: Query<(&Transform, &Vision)>) {
    for (transform, vision) in &query {
        let origin = transform.translation.xy();
        let forward = vision.aim_direction;

        gizmos.arrow_2d(origin, origin + forward * 64.0, GREEN);

        let left = forward.rotate(Vec2::from_angle(45f32.to_radians()));
        let right = forward.rotate(Vec2::from_angle(-45f32.to_radians()));

        gizmos.line_2d(origin, origin + left * 64.0, YELLOW);
        gizmos.line_2d(origin, origin + right * 64.0, YELLOW);
    }
}

// ---------------------
// AGGRO / TARGETING
// ---------------------

/// Handles auto-targeting when an entity is attacked.
/// Ignores line of sight or cone checks — instant rage response.
/// For now, this will just target the entity the AI is watching, not necessarily the entity that damaged them.
/// TODO: Refactor projectiles/melee/damage to hold the "original source" entity so this can be improved
pub fn on_damage_aggro(
    damage_trigger: Trigger<DamageDealtEvent>,
    mut commands: Commands,
    target_query: Query<&Watching>,
) {
    if let Ok(watching) = target_query.get(damage_trigger.target()) {
        debug!(
            "I've been hit: {}, attacking: {}",
            damage_trigger.target(),
            watching.0
        );
        commands
            .entity(damage_trigger.target())
            .insert((TargetLock, Targeting(watching.0)));

        schedule_component_removal::<TargetLock>(&mut commands, damage_trigger.target(), 6.0);
    }
}

/// Starts targeting the watched entity if it is in sight and in the vision cone.
pub fn should_target_watched(
    mut commands: Commands,
    npc_query: Query<(&TargetInfo, &Watching, Entity), Without<Targeting>>,
) {
    npc_query
        .iter()
        .for_each(|(target_info, watching, entity)| {
            if target_info.line_of_sight && target_info.in_vision_cone {
                commands.entity(entity).insert(Targeting(watching.0));
            }
        });
}

/// Stops targeting if the entity loses sight of the target or the target lock has expired.
pub fn should_stop_targeting(
    mut commands: Commands,
    npc_query: Query<(&TargetInfo, Has<TargetLock>, Entity), With<Targeting>>,
) {
    npc_query
        .iter()
        .for_each(|(target_info, has_lock, entity)| {
            if !target_info.line_of_sight && !has_lock {
                commands.entity(entity).remove::<Targeting>();
            }
        });
}
