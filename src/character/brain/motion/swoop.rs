//! The bat's dive through the target and back out.

use bevy::prelude::*;

use super::{
    CurrentBehaviorRunId, DecayingSpeedBoost, MotionStep, SteerInput, SteeredCharacter, Steering,
    report_motion_finished,
};
use crate::character::brain::{Capability, CapabilityProfile, Motion};

const SWOOP_EXTRA_SPEED: f32 = 1.0;
const SWOOP_EXIT_TURN: f32 = 2.0 * std::f32::consts::FRAC_PI_3;
const SWOOP_LOOKAHEAD: f32 = 40.0;
const SWOOP_TRACK_LIMIT: f32 = 0.25;
const SWOOP_TRACK_RATIO: f32 = 0.5;
const SWOOP_ARC_SAMPLES: usize = 12;
const SWOOP_RECOVERY_TIME: f32 = 0.4;

/// A committed curved dive through the target; on ending it hands steering a
/// decaying [`DecayingSpeedBoost`].
#[derive(Component, Clone, Debug)]
pub struct Swoop {
    pub capability_profile: CapabilityProfile,
    pub extra_speed: f32,
    pub exit_turn: f32,
    pass: Option<SwoopPass>,
    current_behavior_run_id: CurrentBehaviorRunId,
}

impl Swoop {
    /// Dives at targets within `reach`, rolling `chance` each time it wins and
    /// waiting `cooldown` seconds after a dive ends.
    pub fn new(chance: f32, reach: f32, cooldown: f32) -> Self {
        Self {
            capability_profile: CapabilityProfile {
                reach: Some(reach),
                chance,
                cooldown,
            },
            extra_speed: SWOOP_EXTRA_SPEED,
            exit_turn: SWOOP_EXIT_TURN,
            pass: None,
            current_behavior_run_id: CurrentBehaviorRunId::default(),
        }
    }

    fn end_pass(&mut self) -> Option<DecayingSpeedBoost> {
        // Cleared or cut short, a pass hands its boost to steering to ease off.
        self.pass.take().map(|_| {
            DecayingSpeedBoost::new(
                self.extra_speed,
                SWOOP_RECOVERY_TIME,
                EaseFunction::SmoothStep,
            )
        })
    }
}

impl Capability for Swoop {
    type Key = Motion;

    fn key(&self) -> Motion {
        Motion::Swoop
    }

    fn capability_profile(&self) -> CapabilityProfile {
        self.capability_profile
    }
}

/// Applies swoop each tick and reports [`MotionFinished`](super::MotionFinished)
/// once far enough away from target
pub(super) fn apply_swoop(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut swoopers: Query<(Entity, SteeredCharacter, &mut Swoop, &mut Steering)>,
) {
    for (entity, character, mut swoop, mut steering) in &mut swoopers {
        if swoop
            .current_behavior_run_id
            .follow_brain_run(character.brain, Motion::Swoop)
            && let Some(decaying_speed_boost) = swoop.end_pass()
        {
            commands.entity(entity).try_insert(decaying_speed_boost);
        }
        let Some(behavior_run_id) = swoop.current_behavior_run_id.get() else {
            continue;
        };
        let Some(input) = character.build_steer_input(Motion::Swoop, &time) else {
            *steering = Steering::HALT;
            continue;
        };
        match swoop.step(&input) {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished {
                decaying_speed_boost,
            } => {
                report_motion_finished(
                    &mut commands,
                    entity,
                    Motion::Swoop,
                    behavior_run_id,
                    decaying_speed_boost,
                );
            }
        }
    }
}

impl Swoop {
    fn step(&mut self, input: &SteerInput) -> MotionStep {
        let speed_multiplier = 1.0 + self.extra_speed;
        let steer = |direction| MotionStep::Steer(Steering::boosted(direction, speed_multiplier));
        let Some(info) = input.target_info else {
            return steer(None);
        };
        let cleared = self
            .pass
            .as_ref()
            .is_some_and(|pass| pass.cleared(input.position, info.distance));
        if cleared {
            let decaying_speed_boost = self.end_pass();
            return MotionStep::Finished {
                decaying_speed_boost,
            };
        }
        if !info.distance.is_finite() {
            return steer(None);
        }
        let flight_distance = input.distance_this_frame(speed_multiplier);
        let heading = input.heading;
        let live_target = info.position;
        let exit_turn = self.exit_turn;
        let pass = self.pass.get_or_insert_with(|| {
            SwoopPass::new(
                input.position,
                live_target,
                info.distance,
                heading,
                exit_turn,
            )
        });
        pass.advance(flight_distance);
        pass.track(live_target, flight_distance * SWOOP_TRACK_RATIO);
        let to_lead = pass.lead(SWOOP_LOOKAHEAD) - input.position;
        steer((to_lead.length_squared() > f32::EPSILON).then(|| to_lead.normalize_or_zero()))
    }
}

#[derive(Clone, Copy, Debug)]
struct SwoopPass {
    start: Vec2,
    control: Vec2,
    end: Vec2,
    tracked_target_position: Vec2,
    initial_target_position: Vec2,
    exit_bearing: Vec2,
    entry_bearing: Vec2,
    exit_radius: f32,
    length: f32,
    progress: f32,
}

impl SwoopPass {
    fn new(start: Vec2, target: Vec2, exit_radius: f32, heading: Vec2, exit_turn: f32) -> Self {
        let entry_bearing = (start - target).normalize_or_zero();
        let heading_side = entry_bearing.perp().dot(heading);
        let side = if heading_side > 0.0 { -1.0 } else { 1.0 };
        let exit_bearing = entry_bearing.rotate(Vec2::from_angle(side * exit_turn));
        let mut pass = Self {
            start,
            control: Vec2::ZERO,
            end: Vec2::ZERO,
            tracked_target_position: target,
            initial_target_position: target,
            exit_bearing,
            entry_bearing,
            exit_radius,
            length: 0.0,
            progress: 0.0,
        };
        pass.rebuild();
        pass
    }

    fn rebuild(&mut self) {
        self.end = self.tracked_target_position + self.exit_bearing * self.exit_radius;
        // Puts the curve's midpoint exactly on the target.
        self.control = self.tracked_target_position * 2.0 - (self.start + self.end) * 0.5;
        self.length = self.estimate_length();
    }

    fn track(&mut self, live_target: Vec2, max_step: f32) {
        let limit = self.exit_radius * SWOOP_TRACK_LIMIT;
        let desired = self.initial_target_position
            + (live_target - self.initial_target_position).clamp_length_max(limit);
        let step = (desired - self.tracked_target_position).clamp_length_max(max_step);
        if step.length_squared() <= f32::EPSILON {
            return;
        }
        self.tracked_target_position += step;
        self.rebuild();
    }

    fn point(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let inverse = 1.0 - t;
        self.start * (inverse * inverse) + self.control * (2.0 * inverse * t) + self.end * (t * t)
    }

    fn estimate_length(&self) -> f32 {
        (1..=SWOOP_ARC_SAMPLES)
            .map(|step| {
                let t0 = (step - 1) as f32 / SWOOP_ARC_SAMPLES as f32;
                let t1 = step as f32 / SWOOP_ARC_SAMPLES as f32;
                self.point(t0).distance(self.point(t1))
            })
            .sum()
    }

    fn advance(&mut self, distance: f32) {
        if self.length > f32::EPSILON {
            self.progress = (self.progress + distance / self.length).clamp(0.0, 1.0);
        } else {
            self.progress = 1.0;
        }
    }

    fn lead(&self, distance: f32) -> Vec2 {
        let lead = if self.length > f32::EPSILON {
            self.progress + distance / self.length
        } else {
            1.0
        };
        self.point(lead)
    }

    fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        if self.progress >= 1.0 {
            return true;
        }
        let passed = (position - self.tracked_target_position).dot(self.entry_bearing) < 0.0;
        passed && target_distance >= self.exit_radius
    }
}
