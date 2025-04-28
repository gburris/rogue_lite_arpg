use rand::{thread_rng, Rng};
use std::ops::Range;

use bevy::{ecs::entity::EntityHashSet, prelude::*};

use crate::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = Behaviors)]
pub struct BehaviorOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = BehaviorOf, linked_spawn)]
pub struct Behaviors(EntityHashSet);

pub struct WanderAnchor {
    origin: Vec2,
    distance: f32,
}

/// Alternates between moving in a random direction and idling
#[derive(Component)]
pub struct Wander {
    /// How long to move in random direction before idling
    move_timer: Timer,
    /// How long to idle before picking a new direction to move
    idle_timer: Timer,
    /// If we want the wander to not move to far from some origin, we add this
    anchor: Option<WanderAnchor>,
}

impl Default for Wander {
    fn default() -> Self {
        Self {
            move_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            idle_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            anchor: None,
        }
    }
}

impl Wander {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn move_timer_range(mut self, move_range: Range<f32>) -> Self {
        let mut rng = thread_rng();
        self.move_timer = Timer::from_seconds(rng.gen_range(move_range), TimerMode::Repeating);
        self
    }

    pub fn idle_timer_range(mut self, idle_range: Range<f32>) -> Self {
        let mut rng = thread_rng();
        self.idle_timer = Timer::from_seconds(rng.gen_range(idle_range), TimerMode::Repeating);
        self
    }

    pub fn anchor(mut self, origin: Vec2, distance: f32) -> Self {
        self.anchor = Some(WanderAnchor { origin, distance });

        self
    }
}

pub fn run_wander(
    time: Res<Time>,
    mut wander_query: Query<(&BehaviorOf, &mut Wander)>,
    mut motion_query: Query<(&mut SimpleMotion, &Transform)>,
) {
    wander_query
        .iter_mut()
        .for_each(|(behavior_of, mut wander)| {
            let (mut motion, transform) = motion_query.get_mut(behavior_of.0).unwrap();
            let delta = time.delta();

            if motion.is_moving() {
                if wander.move_timer.tick(delta).just_finished() {
                    motion.stop_moving();
                }
            } else if wander.idle_timer.tick(delta).just_finished() {
                let direction = wander
                    .anchor
                    .as_ref()
                    .filter(|anchor| {
                        transform.translation.truncate().distance(anchor.origin) > anchor.distance
                    })
                    .map(|anchor| {
                        (anchor.origin - transform.translation.truncate()).normalize_or_zero()
                    })
                    .unwrap_or_else(random_direction);

                motion.start_moving(direction);
            }
        });
}

fn random_direction() -> Vec2 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin())
}
