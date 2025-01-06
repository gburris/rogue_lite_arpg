use crate::components::Speed;
use crate::npc::NPC;
use bevy::prelude::*;

#[derive(Component)]
pub struct NPCMovement {
    start_pos: Vec3,
    direction: f32,
    stand_timer: Timer,
    is_standing: bool,
}

impl Default for NPCMovement {
    fn default() -> Self {
        Self {
            start_pos: Vec3::ZERO,
            direction: 1.0, // Start moving right
            stand_timer: Timer::from_seconds(10.0, TimerMode::Once),
            is_standing: false,
        }
    }
}

pub fn move_npcs(
    time: Res<Time>,
    mut query: Query<(&Speed, &mut Transform, &mut NPCMovement), With<NPC>>,
) {
    for (speed, mut transform, mut movement) in query.iter_mut() {
        // Initialize start position if it's zero
        if movement.start_pos == Vec3::ZERO {
            movement.start_pos = transform.translation;
        }

        // If standing, update timer and check if we should start moving again
        if movement.is_standing {
            movement.stand_timer.tick(time.delta());

            if movement.stand_timer.finished() {
                movement.is_standing = false;
                movement.direction *= -1.0; // Reverse direction
                movement.stand_timer.reset();
                warn!("NPC started walking");
                // You might want to add sprite flipping logic here
            }
            continue; // Skip movement while standing
        }

        // Calculate movement when not standing
        let new_pos = transform.translation.x + (movement.direction * speed.velocity);
        let distance_from_start = (new_pos - movement.start_pos.x).abs();

        // Check if we've reached the movement boundary (50 units)
        if distance_from_start >= 500.0 {
            movement.is_standing = true;
            transform.translation.x = movement.start_pos.x + (500.0 * movement.direction.signum());
            warn!("NPC reached boundary and stopped");
        } else {
            transform.translation.x = new_pos;
        }
    }
}
