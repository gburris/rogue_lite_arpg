use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    ai::{
        state::{ActionState, AimPosition},
        SimpleMotion,
    },
    combat::Health,
    enemy::Enemy,
    items::equipment::EquipmentSlot,
    npc::NPC,
    player::{Player, UseEquipmentInputEvent},
};

#[derive(Component)]
pub struct WanderDirection {
    timer: Timer,
}

impl Default for WanderDirection {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

pub fn update_enemy_aim_position(
    mut enemy_aim_pos_query: Query<&mut AimPosition, With<Enemy>>,
    player_transform: Single<&mut Transform, With<Player>>,
) {
    for mut aim_position in enemy_aim_pos_query.iter_mut() {
        aim_position.position = player_transform.translation.truncate();
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
            &ActionState,
            Option<&mut WanderDirection>,
        ),
        (With<Enemy>, Without<NPC>),
    >,
    player_transform: Single<&Transform, With<Player>>,
) {
    const CHASE_DISTANCE: f32 = 400.0;

    let player_pos = player_transform.translation;

    for (entity, health, enemy_transform, mut motion, state, wander) in enemy_query.iter_mut() {
        if *state == ActionState::Defeated {
            motion.stop_moving();
            continue;
        }
        let distance_to_player = player_pos.distance(enemy_transform.translation);

        if distance_to_player <= CHASE_DISTANCE || health.hp < health.max_hp {
            // Remove wandering component if it exists when in chase mode
            if wander.is_some() {
                commands.entity(entity).remove::<WanderDirection>();
            }

            commands.trigger_targets(
                UseEquipmentInputEvent {
                    slot: EquipmentSlot::Mainhand,
                },
                entity,
            );

            // Chase behavior
            let towards_player_direction = (player_pos - enemy_transform.translation)
                .normalize_or_zero()
                .truncate();
            motion.start_moving(towards_player_direction);
        } else {
            // Wandering behavior
            match wander {
                Some(mut wander) => {
                    // Update wander timer and change direction if needed
                    if wander.timer.tick(time.delta()).just_finished() {
                        motion.start_moving(random_direction());
                    }
                }
                None => {
                    // Initialize wandering for enemies that don't have it
                    motion.start_moving(random_direction());
                    commands.entity(entity).insert(WanderDirection {
                        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                    });
                }
            }
        };
    }
}

fn random_direction() -> Vec2 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin())
}
