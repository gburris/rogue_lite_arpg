use bevy_behave::prelude::BehaveCtx;
use rand::{thread_rng, Rng};
use std::ops::Range;

use bevy::prelude::*;

use crate::{combat::Health, prelude::*};

#[derive(Component, Clone)]
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
    target_query: Query<&Vision>,
) {
    idle_query.iter_mut().for_each(|(ctx, mut idle)| {
        let vision = target_query.get(ctx.target_entity()).unwrap();

        if vision.has_target {
            commands.trigger(ctx.failure());
        } else if idle.timer.tick(time.delta()).just_finished() {
            commands.trigger(ctx.success());
        }
    });
}

#[derive(Clone)]
pub struct WanderAnchor {
    origin: Vec2,
    distance: f32,
}

/// Moves in a random direction
#[derive(Component, Clone)]
pub struct Wander {
    /// How long to move in direction for
    timer: Timer,
    /// If we want the wander to not move to far from some origin, we add this
    anchor: Option<WanderAnchor>,
}

impl Default for Wander {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            anchor: None,
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

    pub fn anchor(mut self, origin: Vec2, distance: f32) -> Self {
        self.anchor = Some(WanderAnchor { origin, distance });

        self
    }
}

pub fn on_wander_start(
    trigger: Trigger<OnAdd, Wander>,
    time: Res<Time>,
    mut wander_query: Query<(&BehaveCtx, &mut Wander)>,
    mut target_query: Query<(&mut SimpleMotion, &Transform)>,
) {
    let (ctx, mut wander) = wander_query.get(trigger.target()).unwrap();

    let (mut motion, transform) = target_query.get_mut(ctx.target_entity()).unwrap();

    let direction = wander
        .anchor
        .as_ref()
        .filter(|anchor| transform.translation.truncate().distance(anchor.origin) > anchor.distance)
        .map(|anchor| (anchor.origin - transform.translation.truncate()).normalize_or_zero())
        .unwrap_or_else(random_direction);

    motion.start_moving(direction);
}

pub fn while_wandering(
    mut commands: Commands,
    time: Res<Time>,
    mut idle_query: Query<(&BehaveCtx, &mut Wander)>,
    target_query: Query<&Vision>,
) {
    idle_query.iter_mut().for_each(|(ctx, mut wander)| {
        let vision = target_query.get(ctx.target_entity()).unwrap();

        if vision.has_target {
            commands.trigger(ctx.failure());
        } else if wander.timer.tick(time.delta()).just_finished() {
            commands.trigger(ctx.success());
        }
    });
}

#[derive(Component, Clone)]
pub struct Chase;

pub fn run_chase(
    mut commands: Commands,
    mut chase_query: Query<(&BehaveCtx, &mut Chase)>,
    mut target_query: Query<(&mut SimpleMotion, &Transform, &Vision)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    chase_query.iter_mut().for_each(|(ctx, _chase)| {
        let (mut motion, target_transform, vision) =
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
            commands.trigger(ctx.success());
        } else if !vision.has_target {
            commands.trigger(ctx.failure());
        }
    });
}

#[derive(Component, Clone)]
pub struct WaitUntilPlayerInSight;

pub fn is_player_in_sight(
    mut commands: Commands,
    behave_query: Query<&BehaveCtx, With<WaitUntilPlayerInSight>>,
    mut enemy_query: Query<(&Health, &Transform), (With<Enemy>, Without<NPC>)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    const VISION_DISTANCE: f32 = 150.0;

    behave_query.iter().for_each(|ctx| {
        let (target_health, target_transform) = enemy_query.get_mut(ctx.target_entity()).unwrap();

        let distance_to_player = player_transform
            .translation
            .xy()
            .distance(target_transform.translation.xy());

        if distance_to_player <= VISION_DISTANCE || target_health.hp < target_health.max_hp {
            commands.trigger(ctx.success());
        } else {
            commands.trigger(ctx.failure());
        }
    });
}

fn random_direction() -> Vec2 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin())
}
