use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    animation::MovementDirection, combat::attributes::Health, enemy::Enemy, movement::components::{IsMoving, SimpleMotion}, player::Player
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
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<
            (
                Entity,
                &Health,
                &Transform,
                &mut SimpleMotion,
                &mut MovementDirection,
                &mut IsMoving,
                Option<&mut WanderDirection>,
            ),
            With<Enemy>,
        >,
    )>,
) {
    const CHASE_DISTANCE: f32 = 400.0;

    if let Ok(player_transform) = param_set.p0().get_single() {
        let player_pos = player_transform.translation;

        for (
            entity,
            health,
            enemy_transform,
            mut motion,
            mut movement_direction,
            mut is_moving,
            wander,
        ) in param_set.p1().iter_mut()
        {
            let distance_to_player = player_pos.distance(enemy_transform.translation);

            let new_direction = if distance_to_player <= CHASE_DISTANCE || health.hp < health.max_hp
            {
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

            // Update movement direction for animations
            if new_direction != Vec3::ZERO {
                movement_direction
                    .set_if_neq(MovementDirection::from_vec2(new_direction.truncate()));
                is_moving.0 = true;
            } else {
                *movement_direction = MovementDirection::None;
                is_moving.0 = false;
            }
        }
    }
}

fn random_direction() -> Vec3 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec3::new(angle.cos(), angle.sin(), 0.0)
}
