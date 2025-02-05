use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{movement::components::SimpleMotion, npc::components::NPC};

#[derive(Component)]
pub struct NPCWanderState {
    origin: Vec3,
    direction: Vec3,
    movement_timer: Timer,
    idle_timer: Timer,
    is_moving: bool,
}

impl Default for NPCWanderState {
    fn default() -> Self {
        let mut rng = thread_rng();
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::ZERO,
            movement_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
            idle_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
            is_moving: false, // Always start idle
        }
    }
}

const TILE_SIZE: f32 = 32.0;
const WANDER_RADIUS: f32 = 2.5 * TILE_SIZE;

pub fn move_npcs(
    time: Res<Time>,
    mut commands: Commands,
    mut npc_query: Query<
        (
            Entity,
            &Transform,
            &mut SimpleMotion,
            Option<&mut NPCWanderState>,
        ),
        With<NPC>,
    >,
) {
    let mut rng = thread_rng();

    for (entity, transform, mut motion, wander_state) in npc_query.iter_mut() {
        match wander_state {
            Some(mut state) => {
                // Update timers
                state.movement_timer.tick(time.delta());
                state.idle_timer.tick(time.delta());

                // Handle state transitions
                if state.is_moving && state.movement_timer.finished() {
                    state.is_moving = false;
                    state.idle_timer =
                        Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once);
                    motion.direction = Vec2::ZERO;
                } else if !state.is_moving && state.idle_timer.finished() {
                    state.is_moving = true;
                    state.movement_timer =
                        Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once);
                    state.direction = random_direction();
                }

                // Check if NPC is too far from origin
                let distance_from_origin = transform.translation.distance(state.origin);
                if distance_from_origin > WANDER_RADIUS {
                    // Move back towards origin
                    state.direction = (state.origin - transform.translation).normalize();
                }

                // Update motion
                if state.is_moving {
                    motion.direction = state.direction.truncate();
                }
            }
            None => {
                // Initialize wandering state for new NPCs
                let state = NPCWanderState {
                    origin: transform.translation,
                    direction: Vec3::ZERO,
                    movement_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
                    idle_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
                    is_moving: false, // Start in idle state
                };
                commands.entity(entity).insert(state);
                // Ensure motion is zeroed initially
                motion.direction = Vec2::ZERO;
            }
        }
    }
}

fn random_direction() -> Vec3 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec3::new(angle.cos(), angle.sin(), 0.0)
}
