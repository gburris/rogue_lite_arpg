use bevy::prelude::*;

use super::{BlockedTimer, DirectMotion, MotionOf, MotionProgress, SteerInput};
use crate::character::state::BlockedState;

/// Seconds a return home may stay blocked before the home moves to where the character stands.
const RETURN_BLOCKED_SECONDS: f32 = 1.0;

/// A character's home, on its own entity placed at the home's center and linked to the character
/// by [`HomeOf`]: how far the character may wander from the center, and how close to it counts as
/// arrived. Motions steer by the entity. If the character is blocked on the way back, the entity
/// moves to where the character stands.
#[derive(Component, Clone, Copy, Debug)]
pub struct HomeArea {
    radius: f32,
    arrival_radius: f32,
}

impl HomeArea {
    pub fn new(radius: f32, arrival_radius: f32) -> Self {
        debug_assert!(
            radius.is_finite() && radius > 0.0,
            "home area radius must be finite and positive"
        );
        debug_assert!(
            arrival_radius > 0.0 && arrival_radius < radius,
            "home area arrival radius must be positive and inside the wander radius"
        );
        Self {
            radius,
            arrival_radius,
        }
    }

    pub(in crate::character) fn radius(self) -> f32 {
        self.radius
    }

    /// Whether `distance` from the center is within the arrival radius.
    pub(in crate::character) fn has_arrived(self, distance: f32) -> bool {
        distance <= self.arrival_radius
    }

    /// How far `distance` from the center reaches beyond the radius; negative while inside it.
    pub(in crate::character) fn overshoot(self, distance: f32) -> f32 {
        distance - self.radius
    }

    /// [`Self::overshoot`] as a fraction of the radius; `None` unless the radius is finite and
    /// positive.
    pub(in crate::character) fn overshoot_fraction(self, distance: f32) -> Option<f32> {
        let radius = self.radius();
        if radius.is_finite() && radius > 0.0 {
            Some(self.overshoot(distance) / radius)
        } else {
            None
        }
    }
}

/// Links a [`HomeArea`] to the character whose home it is.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = Home)]
pub struct HomeOf {
    pub character: Entity,
}

/// The [`HomeArea`] of this character.
#[derive(Component, Debug)]
#[relationship_target(relationship = HomeOf, linked_spawn)]
pub struct Home(Entity);

impl Home {
    pub fn get(&self) -> Entity {
        self.0
    }
}

/// Walks straight back to the character's [`HomeArea`] and stops on arrival.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ReturnToHome;

impl DirectMotion for ReturnToHome {
    fn pick_steering_direction(&self, input: &SteerInput) -> Option<Vec2> {
        input.direction_to_home()
    }
}

/// A return home in progress, on its running motion: how long it has been blocked.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct ReturnToHomeProgress {
    blocked_timer: BlockedTimer,
}

impl MotionProgress for ReturnToHomeProgress {
    type Tuning = ReturnToHome;
}

/// Moves the home of each character whose return home is blocked to where it stands.
pub(super) fn move_homes_of_blocked_characters(
    time: Res<Time<Virtual>>,
    mut running_returns: Query<(&MotionOf, &mut ReturnToHomeProgress)>,
    characters: Query<(&Transform, BlockedState, &Home), Without<HomeArea>>,
    mut homes: Query<&mut Transform, With<HomeArea>>,
) {
    for (motion_of, mut progress) in &mut running_returns {
        let Ok((transform, blocked, home)) = characters.get(motion_of.character) else {
            continue;
        };
        let is_blocked_too_long = progress.blocked_timer.tick_and_check_limit(
            blocked.is_blocked(),
            time.delta_secs(),
            RETURN_BLOCKED_SECONDS,
        );
        if is_blocked_too_long && let Ok(mut home_transform) = homes.get_mut(home.get()) {
            home_transform.translation = transform
                .translation
                .xy()
                .extend(home_transform.translation.z);
        }
    }
}
