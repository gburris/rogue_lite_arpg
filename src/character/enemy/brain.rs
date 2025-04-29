use avian2d::prelude::{RayCaster, RayHits};
use bevy::prelude::*;

use crate::{character::state::Aim, prelude::*};

/// The enemies RayCast represents it's "vision". Here we update where the raycast faces based on player position
pub fn update_enemy_vision(
    mut enemy_query: Query<(&mut Aim, &mut RayCaster, &Transform, &FacingDirection), With<Enemy>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    for (mut aim, mut ray_caster, transform, facing_direction) in enemy_query.iter_mut() {
        // Calculate vector to player
        let to_player: Vec2 = player_transform.translation.xy() - transform.translation.xy();

        // Only face player if relatively in front (within 90 degrees of facing direction)
        let facing_vec = facing_direction.to_vec2();
        if !aim.has_target() {
            aim.position = facing_vec;
        }

        if to_player.normalize_or_zero().dot(facing_vec) > 0.0 {
            // Player is in front - aim at player
            ray_caster.direction = Dir2::new(to_player).unwrap_or(Dir2::X);
        } else {
            // Player is behind - continue facing current direction
            ray_caster.direction = Dir2::new(facing_vec).unwrap_or(Dir2::X);
        }
    }
}

pub fn is_player_in_sight(
    mut enemy_query: Query<(&mut Aim, &RayHits), (With<Enemy>, Without<NPC>)>,
    player: Single<(&Children, &Transform, Entity), With<Player>>,
) {
    let (player_children, player_transform, player) = player.into_inner();
    enemy_query.par_iter_mut().for_each(|(mut aim, ray_hits)| {
        // Default to no target
        aim.target = None;

        // Check all hits (we don't need to sort since we only have max hit of 1)
        for hit in ray_hits.iter() {
            if player_children.contains(&hit.entity) {
                aim.position = player_transform.translation.truncate();
                aim.target = Some(player);
                break; // Found player, no need to check further hits
            }
        }
    });
}
