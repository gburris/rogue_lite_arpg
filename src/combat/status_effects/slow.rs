use bevy::prelude::*;

use super::freeze::Frozen;
use super::status::{StatusKind, Statuses};
use crate::prelude::*;

#[derive(Component, Clone, Copy)]
#[require(StatusKind::Slow)]
pub struct Slowed {
    pub percent: f32,
}

impl Default for Slowed {
    fn default() -> Self {
        Self { percent: 0.5 }
    }
}

/// The fraction of a character's speed that its statuses remove. The character feature applies
/// it to the character's [`Kinematics`].
#[derive(Component, Clone, Copy, Debug, Deref)]
#[component(immutable)]
pub struct MovementSlow(pub f32);

/// Sets each character's [`MovementSlow`] to its strongest active status, with a freeze
/// counting as a full stun, so one status expiring never clears another.
pub(super) fn sync_slow_from_statuses(
    mut commands: Commands,
    characters: Query<(Entity, Option<&Statuses>, Option<&MovementSlow>), With<Kinematics>>,
    slows: Query<&Slowed>,
    freezes: Query<(), With<Frozen>>,
) {
    for (character, statuses, current_slow) in &characters {
        let strongest = statuses
            .into_iter()
            .flat_map(Statuses::iter)
            .map(|status| {
                if freezes.contains(status) {
                    1.0
                } else {
                    slows.get(status).map_or(0.0, |slowed| slowed.percent)
                }
            })
            .fold(0.0, f32::max);
        let current = current_slow.map_or(0.0, |slow| **slow);
        if (current - strongest).abs() > f32::EPSILON {
            commands
                .entity(character)
                .try_insert(MovementSlow(strongest));
        }
    }
}
