//! Aimless drifting inside the anchor.

use bevy::prelude::*;

use super::SteerInput;
use crate::character::brain::BrainRng;

/// Wander speed as a fraction of max speed; ambling keeps steps inside small anchors.
const WANDER_SPEED_SCALE: f32 = 0.4;
/// Seconds between direction rerolls when a close target leaves no bearing.
const JITTER_INTERVAL: f32 = 0.2;

/// How one entity wanders. Opt-in at spawn; absent means that entity never wanders.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    pub min: f32,
    pub max: f32,
    pub speed_scale: f32,
    direction: Vec2,
    next_direction_at: f32,
}

impl Wander {
    /// Holds each direction for a uniform roll in `min..=max` seconds.
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            speed_scale: WANDER_SPEED_SCALE,
            direction: Vec2::ZERO,
            next_direction_at: 0.0,
        }
    }

    /// A short-lived random heading, used to peel off a target the body is
    /// sitting directly on.
    pub(super) fn jitter(&mut self, now: f32, random: &mut BrainRng) -> Vec2 {
        if now >= self.next_direction_at {
            self.direction = random.direction();
            self.next_direction_at = now + JITTER_INTERVAL;
        }
        self.direction
    }
}

/// Speed scale to wander at, with the default when the entity carries no `Wander`.
pub(super) fn speed_scale(wander: Option<&Wander>) -> f32 {
    wander.map_or(WANDER_SPEED_SCALE, |wander| wander.speed_scale)
}

/// Holds a rolled direction, steering back toward the anchor near its edge.
pub(super) fn steer(
    input: &SteerInput,
    wander: Option<&mut Wander>,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let wander = wander?;
    if input.now >= wander.next_direction_at {
        wander.direction = pick_direction(input, random);
        wander.next_direction_at = input.now + random.range(wander.min, wander.max);
    }
    Some(wander.direction)
}

fn pick_direction(input: &SteerInput, random: &mut BrainRng) -> Vec2 {
    let candidate = random.direction();
    let Some(anchor) = input.anchor else {
        return candidate;
    };
    if anchor
        .overshoot_fraction(input.position)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let home = anchor.origin() - input.position;
    if home == Vec2::ZERO {
        return candidate;
    }
    home.normalize_or_zero()
        .rotate(Vec2::from_angle(random.range(-0.6, 0.6)))
}
