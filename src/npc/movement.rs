use bevy::prelude::*;

use crate::{movement::components::SimpleMotion, npc::NPC};

#[derive(Component)]
pub struct NPCMovement {
    start_pos: Vec3,
    stand_timer: Timer,
    is_standing: bool,
}

impl Default for NPCMovement {
    fn default() -> Self {
        Self {
            start_pos: Vec3::ZERO,
            stand_timer: Timer::from_seconds(1.0, TimerMode::Once),
            is_standing: false,
        }
    }
}

pub fn move_npcs(
    time: Res<Time>,
    mut query: Query<(&Transform, &mut SimpleMotion, &mut NPCMovement), With<NPC>>,
) {
    for (transform, mut simple_motion, mut npc_movement) in query.iter_mut() {
        // Initialize start position if it's zero
        if npc_movement.start_pos == Vec3::ZERO {
            npc_movement.start_pos = transform.translation;
        }

        // If standing, update timer and check if we should start moving again
        if npc_movement.is_standing {
            npc_movement.stand_timer.tick(time.delta());

            if npc_movement.stand_timer.finished() {
                npc_movement.is_standing = false;
                simple_motion.direction = Vec2::new(simple_motion.direction.x * -1.0, 0.0); // Reverse direction
                npc_movement.stand_timer.reset();
                npc_movement.start_pos = transform.translation;
                simple_motion.can_move = true;
            }
        } else {
            let distance_from_start = (transform.translation.x - npc_movement.start_pos.x).abs();

            trace!(
                "NPC is moving, distance from start: {}, speed: {}, direction: {}",
                distance_from_start,
                simple_motion.current_speed,
                simple_motion.direction
            );

            // Check if we've reached the movement boundary (50 units)
            if distance_from_start >= 500.0 {
                npc_movement.is_standing = true;
                simple_motion.can_move = false;
            }
        }
    }
}
