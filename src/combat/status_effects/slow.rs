use bevy::prelude::*;

use super::freeze::Frozen;
use super::{StatusType, Statuses};
use crate::prelude::Kinematics;

#[derive(Component, Clone)]
#[require(StatusType::Slow)]
pub struct Slowed {
    pub percent: f32,
}

impl Default for Slowed {
    fn default() -> Self {
        Slowed { percent: 0.5 }
    }
}

/// Sets each character's slow to its strongest active status, with a freeze
/// counting as a full stun, so one status expiring never clears another.
pub(crate) fn sync_slow_from_statuses(
    mut characters: Query<(&mut Kinematics, Option<&Statuses>)>,
    slows: Query<&Slowed>,
    freezes: Query<(), With<Frozen>>,
) {
    for (mut kinematics, statuses) in &mut characters {
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
        if (kinematics.slowed_percentage() - strongest).abs() > f32::EPSILON {
            kinematics.slow(strongest);
        }
    }
}
