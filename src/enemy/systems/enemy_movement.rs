use crate::components::{Enemy, Health, Speed};
use crate::player::components::Player;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

// Add this to your components module
#[derive(Component)]
pub struct WanderDirection {
    direction: Vec3,
    timer: Timer,
}

impl Default for WanderDirection {
    fn default() -> Self {
        Self {
            direction: Vec3::ZERO,
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

pub fn move_enemies_toward_player(
    time: Res<Time>,
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<
            (
                Entity,
                &Speed,
                &Health,
                &mut Transform,
                Option<&mut WanderDirection>,
            ),
            With<Enemy>,
        >,
    )>,
) {
    const CHASE_DISTANCE: f32 = 400.0;

    // Get the player's transform (read-only)
    if let Ok(player_transform) = param_set.p0().get_single() {
        let player_pos = player_transform.translation;

        // Iterate through enemies and update their transforms
        for (entity, speed, healh, mut enemy_transform, wander) in param_set.p1().iter_mut() {
            let distance_to_player = player_pos.distance(enemy_transform.translation);

            if distance_to_player <= CHASE_DISTANCE || healh.hp < healh.max_hp {
                // Remove wandering component if it exists when in chase mode
                if wander.is_some() {
                    commands.entity(entity).remove::<WanderDirection>();
                }

                // Chase behavior
                let direction = (player_pos - enemy_transform.translation).normalize_or_zero();
                enemy_transform.translation.x += direction.x * speed.velocity;
                enemy_transform.translation.y += direction.y * speed.velocity;
            } else {
                // Wandering behavior
                match wander {
                    Some(mut wander) => {
                        // Update wander timer and change direction if needed
                        if wander.timer.tick(time.delta()).just_finished() {
                            wander.direction = random_direction();
                        }
                        enemy_transform.translation.x += wander.direction.x * speed.velocity * 0.5;
                        enemy_transform.translation.y += wander.direction.y * speed.velocity * 0.5;
                    }
                    None => {
                        // Initialize wandering for enemies that don't have it
                        commands.entity(entity).insert(WanderDirection {
                            direction: random_direction(),
                            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                        });
                    }
                }
            }
        }
    }
}

fn random_direction() -> Vec3 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec3::new(angle.cos(), 0.0, 0.5)
}
