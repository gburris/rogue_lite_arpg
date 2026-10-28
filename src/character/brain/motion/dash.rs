//! Committed straight runs: the warrior's kite, the fire mage's hop and the ice mage's slide.

use bevy::prelude::*;

use super::{MotionStep, RunTag, SteerInput, SteeredCharacter, Steering, report_motion_finished};
use crate::character::brain::perception::{TargetView, build_target_view};
use crate::character::brain::{BrainRng, Capability, Motion, Tuning};

/// Speed below which a target counts as holding still rather than orbiting.
const TARGET_MOVING_SPEED: f32 = 20.0;

const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

const HOP_STEP_IN_SPREAD: f32 = 0.35;
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// A committed straight run along a direction fixed at its start.
#[derive(Component, Clone, Debug)]
pub struct Dash {
    pub min_run: f32,
    pub max_run: f32,
    pub speed_boost: f32,
    pub tuning: Tuning,
    aim: AimKind,
    debug_name: &'static str,
    run: StraightRun,
    tag: RunTag,
}

impl Dash {
    fn with_aim(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        tuning: Tuning,
        aim: AimKind,
        debug_name: &'static str,
    ) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            tuning,
            aim,
            debug_name,
            run: StraightRun::default(),
            tag: RunTag::default(),
        }
    }

    pub fn kite(chance: f32, min_run: f32, max_run: f32) -> Self {
        Self::with_aim(
            min_run,
            max_run,
            1.0,
            Tuning {
                chance,
                ..Tuning::default()
            },
            AimKind::Away {
                spread: KITE_SPREAD,
            },
            "Kite",
        )
    }

    pub fn hop(min_run: f32, max_run: f32, speed_boost: f32, close_range: f32) -> Self {
        Self::with_aim(
            min_run,
            max_run,
            speed_boost,
            Tuning::default(),
            AimKind::HopTangent { close_range },
            "Hop",
        )
    }

    pub fn slide(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        danger_range: f32,
        cooldown: f32,
    ) -> Self {
        Self::with_aim(
            min_run,
            max_run,
            speed_boost,
            Tuning {
                reach: Some(danger_range),
                cooldown,
                ..Tuning::default()
            },
            AimKind::Away { spread: 0.0 },
            "Slide",
        )
    }

    pub fn name(&self) -> &'static str {
        self.debug_name
    }
}

fn hop_direction(target: &TargetView, close_range: f32, heading: Vec2, rng: &mut BrainRng) -> Vec2 {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let spread = rng.range(-HOP_STEP_IN_SPREAD, HOP_STEP_IN_SPREAD);
        return target.bearing.rotate(Vec2::from_angle(spread));
    }
    // Cut across to the side opposite the current heading, so an orbit that
    // follows reads the new heading and circles back the other way.
    let heading_side = target.bearing.perp().dot(heading);
    let tangent_sign = if heading_side.abs() > f32::EPSILON {
        -heading_side.signum()
    } else if rng.range(0.0, 1.0) < 0.5 {
        1.0
    } else {
        -1.0
    };
    let tangent = target.bearing.perp() * tangent_sign;
    (tangent - target.bearing * HOP_OUTWARD_BIAS).normalize_or_zero()
}

impl Capability for Dash {
    type Key = Motion;

    fn key(&self) -> Motion {
        Motion::Dash
    }

    fn tuning(&self) -> Tuning {
        self.tuning
    }
}

/// Runs a held dash and reports [`MotionFinished`](super::MotionFinished) at
/// the end of its distance.
pub(super) fn apply_dash(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut dashers: Query<(
        Entity,
        SteeredCharacter,
        &mut Dash,
        &mut Steering,
        &mut BrainRng,
    )>,
) {
    for (entity, character, mut dash, mut steering, mut rng) in &mut dashers {
        if dash.tag.follow_brain_run(character.brain, Motion::Dash) {
            dash.run.finish();
        }
        let Some(epoch) = dash.tag.current_epoch() else {
            continue;
        };
        let Some(input) = character.build_steer_input(Motion::Dash, &time) else {
            *steering = Steering::HALT;
            continue;
        };
        match dash.step(&input, &mut rng) {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished { speed_boost } => {
                report_motion_finished(&mut commands, entity, Motion::Dash, epoch, speed_boost);
            }
        }
    }
}

impl Dash {
    fn step(&mut self, input: &SteerInput, rng: &mut BrainRng) -> MotionStep {
        if !self.run.called {
            // A target with no finite distance gives nothing to aim at, so this
            // waits rather than aiming blind.
            let Some(target) = build_target_view(true, input.target_info, input.now) else {
                return MotionStep::Steer(Steering::boosted(None, self.speed_boost));
            };
            let direction = self.aim.aim(target, input.heading, rng);
            let run = rng.range(self.min_run, self.max_run);
            self.run.call(direction, run);
        }
        let distance = input.distance_this_frame(self.speed_boost);
        match self.run.step(distance) {
            Some(direction) => {
                MotionStep::Steer(Steering::boosted(Some(direction), self.speed_boost))
            }
            None => MotionStep::Finished { speed_boost: None },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AimKind {
    Away { spread: f32 },
    HopTangent { close_range: f32 },
}

impl AimKind {
    fn aim(self, target: TargetView, heading: Vec2, rng: &mut BrainRng) -> Vec2 {
        let direction = match self {
            Self::Away { spread } => (-target.bearing)
                .normalize_or_zero()
                .rotate(Vec2::from_angle(rng.range(-spread, spread))),
            Self::HopTangent { close_range } => hop_direction(&target, close_range, heading, rng),
        };
        // A zero bearing (the characters overlap) has no direction to aim along.
        if direction.length_squared() > f32::EPSILON {
            direction
        } else {
            rng.direction()
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct StraightRun {
    direction: Vec2,
    remaining: f32,
    called: bool,
}

impl StraightRun {
    fn call(&mut self, direction: Vec2, run: f32) {
        self.direction = direction;
        self.remaining = run.max(0.0);
        self.called = true;
    }

    fn finish(&mut self) {
        self.called = false;
        self.remaining = 0.0;
    }

    fn step(&mut self, distance: f32) -> Option<Vec2> {
        if !self.called {
            return None;
        }
        if distance >= self.remaining {
            self.finish();
            return None;
        }
        self.remaining -= distance;
        Some(self.direction)
    }
}
