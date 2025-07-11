use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};
use rand::{thread_rng, Rng};
use std::ops::Range;

use bevy::prelude::*;

use crate::{
    character::vision::{TargetInfo, Targeting},
    combat::melee::{ActiveMeleeAttack, MeleeWeapon},
    items::equipment::{EquipmentSlot, Equippable, UseEquipmentInputEvent},
    prelude::*,
};

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
    target_query: Query<&Targeting>,
) {
    idle_query.iter_mut().for_each(|(ctx, mut idle)| {
        if let Ok(_) = target_query.get(ctx.target_entity()) {
            info!("{} Got target while idling", ctx.target_entity());
            commands.trigger(ctx.failure());
        } else if idle.timer.tick(time.delta()).just_finished() {
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
    target_query: Query<&Targeting>,
) {
    wander_query.iter_mut().for_each(|(ctx, mut wander)| {
        if let Ok(_) = target_query.get(ctx.target_entity()) {
            info!("{} Got target while wandering", ctx.target_entity());
            commands.trigger(ctx.failure());
        } else if wander.timer.tick(time.delta()).just_finished() {
            commands.trigger(ctx.success());
        }
    });
}

/// When a character is not agroed and too far from home, return to origin
#[derive(Component, Clone)]
pub struct Retreat;

pub fn while_retreating(
    mut commands: Commands,
    mut retreat_query: Query<&BehaveCtx, With<Retreat>>,
    mut target_query: Query<(&mut SimpleMotion, &Transform, &Anchor, Has<Targeting>)>,
) {
    retreat_query.iter_mut().for_each(|ctx| {
        let (mut motion, transform, anchor, has_target) =
            target_query.get_mut(ctx.target_entity()).unwrap();
        if has_target {
            commands.trigger(ctx.failure());
            // within half a tile, we can stop retreating
        } else if anchor.distance_from(transform) < 16.0 {
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
    mut chase_query: Query<&BehaveCtx, With<Chase>>,
    mut target_query: Query<(&mut SimpleMotion, &TargetInfo, Has<Targeting>)>,
) {
    chase_query.iter_mut().for_each(|ctx| {
        let (mut motion, target_info, has_target) =
            target_query.get_mut(ctx.target_entity()).unwrap();

        if !has_target {
            debug!("We chased and failed!");
            commands.trigger(ctx.failure());
        } else {
            motion.start_moving(target_info.direction);

            if target_info.distance < 64.0 {
                debug!("We chased and succeeded!");
                commands.trigger(ctx.success());
            }
        }
    });
}

fn random_direction() -> Vec2 {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    Vec2::new(angle.cos(), angle.sin())
}

#[derive(Clone)]
pub struct AttemptMelee;

pub fn on_attempt_melee(trigger: Trigger<BehaveTrigger<AttemptMelee>>, mut commands: Commands) {
    let ctx = trigger.ctx();

    commands.trigger_targets(
        UseEquipmentInputEvent {
            slot: EquipmentSlot::Mainhand,
        },
        ctx.target_entity(),
    );
    commands.trigger(ctx.success());
}

#[derive(Component, Clone)]
pub struct KeepDistanceAndFire;

pub fn while_keeping_distance_and_firing(
    mut commands: Commands,
    mut behave_query: Query<&BehaveCtx, With<KeepDistanceAndFire>>,
    mut target_query: Query<(&mut SimpleMotion, &TargetInfo, Has<Targeting>)>,
) {
    behave_query.iter_mut().for_each(|ctx| {
        let (mut motion, target_info, has_target) =
            target_query.get_mut(ctx.target_entity()).unwrap();

        if !has_target {
            commands.trigger(ctx.failure());
        } else {
            commands.trigger_targets(
                UseEquipmentInputEvent {
                    slot: EquipmentSlot::Mainhand,
                },
                ctx.target_entity(),
            );

            if target_info.distance < 200.0 {
                // If target is too close we try to move away
                // TODO: Make this a variable on a component
                motion.start_moving(-target_info.direction);
            } else if target_info.distance > 300.0 {
                motion.start_moving(target_info.direction);
            }
        }
    });
}
