use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    combat::attributes::Health, enemy::Enemy, movement::components::SimpleMotion, player::Player,
};

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
    mut enemy_query: Query<
        (
            Entity,
            &Health,
            &Transform,
            &mut SimpleMotion,
            Option<&mut WanderDirection>,
        ),
        With<Enemy>,
    >,
    player_transform: Single<&Transform, With<Player>>,
) {
    const CHASE_DISTANCE: f32 = 400.0;

    let player_pos = player_transform.translation;

    for (entity, health, enemy_transform, mut motion, wander) in enemy_query.iter_mut() {
        let distance_to_player = player_pos.distance(enemy_transform.translation);

        let new_direction = if distance_to_player <= CHASE_DISTANCE || health.hp < health.max_hp {
            // Remove wandering component if it exists when in chase mode
            if wander.is_some() {
                commands.entity(entity).remove::<WanderDirection>();
            }

            // Chase behavior
            (player_pos - enemy_transform.translation).normalize_or_zero()
        } else {
            // Wandering behavior
            match wander {
                Some(mut wander) => {
                    // Update wander timer and change direction if needed
                    if wander.timer.tick(time.delta()).just_finished() {
                        wander.direction = random_direction();
                    }
                    wander.direction
                }
                None => {
                    // Initialize wandering for enemies that don't have it
                    let direction = random_direction();
                    commands.entity(entity).insert(WanderDirection {
                        direction,
                        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                    });
                    direction
                }
            }
        };

        motion.direction = new_direction.truncate();
    }
}

fn random_direction() -> Vec3 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec3::new(angle.cos(), angle.sin(), 0.0)
}
