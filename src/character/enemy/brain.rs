use avian2d::prelude::{RayCaster, RayHits};
use bevy::{
    color::palettes::css::{GREEN, YELLOW},
    prelude::*,
};

use crate::{character::vision::Agro, combat::damage::DamageDealtEvent, prelude::*};

/// The enemies RayCast represents it's "vision". Here we update where the raycast faces based on player position
pub fn point_raycast_to_player(
    mut enemy_query: Query<(&mut RayCaster, &Transform), With<Enemy>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    for (mut ray_caster, transform) in enemy_query.iter_mut() {
        // Calculate vector to player
        let to_player: Vec2 = player_transform.translation.xy() - transform.translation.xy();

        // point raycast in direction of player
        ray_caster.direction = Dir2::new(to_player).unwrap_or(Dir2::X);
    }
}

pub fn is_player_in_sight(
    mut enemy_query: Query<(&mut Agro, &RayHits)>,
    player: Single<&Children, With<Player>>,
) {
    let player_children = player.into_inner();
    enemy_query.par_iter_mut().for_each(|(mut agro, ray_hits)| {
        agro.line_of_sight = false;

        // Check all hits (we don't need to sort since we only have max hit of 1)
        for hit in ray_hits.iter() {
            if player_children.contains(&hit.entity) {
                agro.line_of_sight = true;
                break; // Found player, no need to check further hits
            }
        }
    });
}

pub fn should_agro_player(
    mut enemy_query: Query<(&mut Agro, &Vision, &Transform)>,
    player: Single<(&Transform, Entity), With<Player>>,
) {
    let (player_transform, player) = player.into_inner();

    enemy_query
        .par_iter_mut()
        .for_each(|(mut agro, vision, transform)| {
            let player_dir = (player_transform.translation.xy() - transform.translation.xy())
                .normalize_or_zero();

            // Calculate dot product: only agro if player is within 45° each side
            let in_vision_cone = player_dir.dot(vision.aim_direction) > 0.707;

            // Update agro based on conditions
            if agro.line_of_sight && in_vision_cone {
                agro.target = Some(player);
            } else if !agro.line_of_sight || agro.target_lock_timer.is_none() {
                agro.target = None;
            }
        });
}

pub fn debug_vision(mut gizmos: Gizmos, query: Query<(&Transform, &Vision)>) {
    for (transform, vision) in &query {
        // Draw vision cone
        gizmos.arrow_2d(
            transform.translation.xy(),
            transform.translation.xy() + (64.0 * vision.aim_direction),
            GREEN,
        );

        // Draw vision cone angles
        let forward = vision.aim_direction;
        let left = forward.rotate(Vec2::from_angle(45f32.to_radians()));
        let right = forward.rotate(Vec2::from_angle(-45f32.to_radians()));

        gizmos.line_2d(
            transform.translation.xy(),
            transform.translation.xy() + left * 64.0,
            YELLOW,
        );
        gizmos.line_2d(
            transform.translation.xy(),
            transform.translation.xy() + right * 64.0,
            YELLOW,
        );
    }
}

pub fn update_aim_position(
    mut character_query: Query<
        (&mut Vision, Option<&Agro>, &FacingDirection, &Transform),
        Without<Player>,
    >,
    target_query: Query<&Transform>,
) {
    for (mut vision, agro, facing_dir, transform) in character_query.iter_mut() {
        vision.aim_direction = agro
            .and_then(|a| a.target)
            .and_then(|target_entity| target_query.get(target_entity).ok())
            .map(|target_transform| {
                (target_transform.translation.xy() - transform.translation.xy()).normalize_or_zero()
            })
            .unwrap_or_else(|| facing_dir.to_vec2());
    }
}

pub fn on_damage_agro(damage_trigger: Trigger<DamageDealtEvent>, mut agro_query: Query<&mut Agro>) {
    if let (Ok(mut agro), Some(source)) = (
        agro_query.get_mut(damage_trigger.target()),
        damage_trigger.damage_source,
    ) {
        agro.lock_target(source);
    }
}

pub fn tick_agro_target_lock(time: Res<Time>, mut agro_query: Query<&mut Agro>) {
    agro_query.par_iter_mut().for_each(|mut agro| {
        let agro_finished = agro
            .target_lock_timer
            .as_mut()
            .map(|t| t.tick(time.delta()).finished())
            .unwrap_or(false);

        if agro_finished {
            agro.target_lock_timer = None;
        }
    });
}
