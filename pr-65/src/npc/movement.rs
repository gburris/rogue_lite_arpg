use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{ai::SimpleMotion, enemy::Enemy, npc::components::NPC};

#[derive(Component)]
pub struct NPCWanderState {
    origin: Vec3,
    movement_timer: Timer,
    idle_timer: Timer,
}

impl Default for NPCWanderState {
    fn default() -> Self {
        let mut rng = thread_rng();
        Self {
            origin: Vec3::ZERO,
            movement_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
            idle_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
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
        (With<NPC>, Without<Enemy>),
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
                if motion.is_moving() && state.movement_timer.finished() {
                    state.idle_timer =
                        Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once);
                    motion.stop_moving();
                } else if !motion.is_moving() && state.idle_timer.finished() {
                    motion.start_moving(random_direction());
                    state.movement_timer =
                        Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once);
                }

                // Check if NPC is too far from origin
                let distance_from_origin = transform.translation.distance(state.origin);
                if distance_from_origin > WANDER_RADIUS {
                    // Move back towards origin
                    let new_direction = (state.origin - transform.translation)
                        .normalize()
                        .truncate();

                    motion.start_moving(new_direction);
                }
            }
            None => {
                // Initialize wandering state for new NPCs
                let state = NPCWanderState {
                    origin: transform.translation,
                    movement_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
                    idle_timer: Timer::from_seconds(rng.gen_range(3.0..10.0), TimerMode::Once),
                };
                commands.entity(entity).insert(state);
            }
        }
    }
}

fn random_direction() -> Vec2 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin())
}
