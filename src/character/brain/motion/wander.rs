//! Calm ambling inside the anchor: stand, walk one heading, stand again.

use bevy::prelude::*;

use super::SteerInput;
use crate::character::brain::{BrainRng, Interval};

/// Wander speed as a fraction of max speed; ambling keeps steps inside small anchors.
const WANDER_SPEED_SCALE: f32 = 0.4;
/// Seconds between direction rerolls when a close target leaves no bearing.
const JITTER_INTERVAL: f32 = 0.2;

/// How one entity spends calm time. Opt-in at spawn; absent means that entity
/// stands still whenever it is calm.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    /// Seconds to stand between walks.
    pub pause: Interval,
    /// Seconds each walk lasts, on one rolled heading.
    pub walk: Interval,
    pub speed_scale: f32,
    direction: Vec2,
    next_direction_at: f32,
    /// The current stretch of the calm rhythm; `None` until calm starts.
    stretch: Option<Stretch>,
}

/// One stretch of the calm rhythm: standing or walking until a time.
#[derive(Clone, Copy, Debug)]
struct Stretch {
    walking: bool,
    until: f32,
}

impl Wander {
    /// Alternates standing for a `pause` roll and walking for a `walk` roll,
    /// always starting with a pause.
    pub fn new(pause: Interval, walk: Interval) -> Self {
        Self {
            pause,
            walk,
            speed_scale: WANDER_SPEED_SCALE,
            direction: Vec2::ZERO,
            next_direction_at: 0.0,
            stretch: None,
        }
    }

    /// Whether the calm rhythm is currently in a walk.
    pub fn is_walking(&self) -> bool {
        self.stretch.is_some_and(|stretch| stretch.walking)
    }

    /// Forgets the rhythm so the next calm spell starts with a fresh pause.
    pub(super) fn leave_calm(&mut self) {
        self.stretch = None;
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

/// Advances the calm rhythm and returns the walk heading, or `None` while
/// standing. Each walk rolls one heading, steering back toward the anchor
/// near its edge.
pub(super) fn steer(
    input: &SteerInput,
    wander: Option<&mut Wander>,
    random: &mut BrainRng,
) -> Option<Vec2> {
    let wander = wander?;
    let stretch = match wander.stretch {
        Some(stretch) if input.now < stretch.until => stretch,
        previous => {
            let walking = previous.is_some_and(|stretch| !stretch.walking);
            let span = if walking { wander.walk } else { wander.pause };
            if walking {
                wander.direction = pick_direction(input, random);
            }
            let stretch = Stretch {
                walking,
                until: input.now + random.range(span.min(), span.max()),
            };
            wander.stretch = Some(stretch);
            stretch
        }
    };
    stretch.walking.then_some(wander.direction)
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
