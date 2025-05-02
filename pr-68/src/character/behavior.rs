use bevy_behave::prelude::BehaveCtx;
use rand::{thread_rng, Rng};
use std::ops::Range;

use bevy::prelude::*;

use crate::prelude::*;

use super::{Agro, Character};

#[derive(Component, Clone)]
#[require(CanBeInterrupted)]
pub struct Idle {
    timer: Timer,
}

impl Default for Idle {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

impl Idle {
    pub fn timer_range(mut self, idle_range: Range<f32>) -> Self {
        let mut rng = thread_rng();
        self.timer = Timer::from_seconds(rng.gen_range(idle_range), TimerMode::Repeating);
        self
    }
}

pub fn on_idle_start(
    trigger: Trigger<OnAdd, Idle>,
    idle_query: Query<&BehaveCtx, With<Idle>>,
    mut target_query: Query<&mut SimpleMotion>,
) {
    let ctx = idle_query.get(trigger.target()).unwrap();

    let mut motion = target_query.get_mut(ctx.target_entity()).unwrap();
    motion.stop_moving();
}

pub fn while_idling(
    mut commands: Commands,
    time: Res<Time>,
    mut idle_query: Query<(&BehaveCtx, &mut Idle)>,
) {
    idle_query.iter_mut().for_each(|(ctx, mut idle)| {
        if idle.timer.tick(time.delta()).just_finished() {
            commands.trigger(ctx.success());
        }
    });
}

/// If we want the character to have a "home base" they return to, add this
#[derive(Component)]
pub struct Anchor {
    origin: Vec2,
    distance: f32,
}

impl Anchor {
    pub fn new(origin: Vec2, distance: f32) -> Self {
        Self { origin, distance }
    }

    pub fn distance_from(&self, transform: &Transform) -> f32 {
        self.origin.distance(transform.translation.xy())
    }

    pub fn outside_range(&self, transform: &Transform) -> bool {
        self.distance_from(transform) > self.distance
    }
}

/// Moves in a random direction
#[derive(Component, Clone)]
#[require(CanBeInterrupted)]
pub struct Wander {
    /// How long to move in direction for
    timer: Timer,
}

impl Default for Wander {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

impl Wander {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn timer_range(mut self, move_range: Range<f32>) -> Self {
        let mut rng = thread_rng();
        self.timer = Timer::from_seconds(rng.gen_range(move_range), TimerMode::Repeating);
        self
    }
}

pub fn on_wander_start(
    trigger: Trigger<OnAdd, Wander>,
    mut commands: Commands,
    wander_query: Query<&BehaveCtx, With<Wander>>,
    mut target_query: Query<(&mut SimpleMotion, Option<&Anchor>, &Transform)>,
) {
    let ctx = wander_query.get(trigger.target()).unwrap();
    let (mut motion, anchor, transform) = target_query.get_mut(ctx.target_entity()).unwrap();

    if anchor.map_or(false, |a| a.outside_range(transform)) {
        commands.trigger(ctx.failure());
    } else {
        motion.start_moving(random_direction());
    }
}

pub fn while_wandering(
    time: Res<Time>,
    mut commands: Commands,
    mut wander_query: Query<(&BehaveCtx, &mut Wander)>,
) {
    wander_query.iter_mut().for_each(|(ctx, mut wander)| {
        if wander.timer.tick(time.delta()).just_finished() {
            commands.trigger(ctx.success());
        }
    });
}

/// Spawn this with a behavior if receiving a target can kick you out of the state
#[derive(Component, Clone, Default)]
pub struct CanBeInterrupted;

pub fn check_for_target_interrupt(
    mut commands: Commands,
    behavior_query: Query<&BehaveCtx, With<CanBeInterrupted>>,
    target_query: Query<Option<&Agro>, With<Character>>,
) {
    behavior_query.iter().for_each(|ctx| {
        let agro = target_query.get(ctx.target_entity()).unwrap();

        if agro.is_some_and(|a| a.has_target()) {
            info!("Interrupting, target found");
            commands.trigger(ctx.failure());
        }
    });
}

/// When a character is not agroed and too far from home, return to origin
#[derive(Component, Clone)]
#[require(CanBeInterrupted)]
pub struct Retreat;

pub fn while_retreating(
    mut commands: Commands,
    mut retreat_query: Query<&BehaveCtx, With<Retreat>>,
    mut target_query: Query<(&mut SimpleMotion, &Transform, &Anchor)>,
) {
    retreat_query.iter_mut().for_each(|ctx| {
        let (mut motion, transform, anchor) = target_query.get_mut(ctx.target_entity()).unwrap();

        // within half a tile, we can stop retreating
        if anchor.distance_from(transform) < 16.0 {
            commands.trigger(ctx.success());
        } else {
            let direction = (anchor.origin - transform.translation.xy()).normalize_or_zero();
            motion.start_moving(direction);
        }
    });
}

/// When a character has a target, it moves towards them. The chase!!
#[derive(Component, Clone)]
pub struct Chase;

pub fn while_chasing(
    mut commands: Commands,
    mut chase_query: Query<(&BehaveCtx, &mut Chase)>,
    mut target_query: Query<(&mut SimpleMotion, &Transform, &Agro)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    chase_query.iter_mut().for_each(|(ctx, _chase)| {
        let (mut motion, target_transform, agro) =
            target_query.get_mut(ctx.target_entity()).unwrap();

        let distance_to_player = player_transform
            .translation
            .xy()
            .distance(target_transform.translation.xy());

        let towards_player_direction = (player_transform.translation.xy()
            - target_transform.translation.xy())
        .normalize_or_zero();

        motion.start_moving(towards_player_direction);

        if distance_to_player < 10.0 {
            info!("We chased and succeeded!");
            commands.trigger(ctx.success());
        } else if !agro.has_target() {
            info!("We chased and failed!");
            commands.trigger(ctx.failure());
        }
    });
}

fn random_direction() -> Vec2 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin())
}
