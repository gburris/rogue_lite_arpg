use bevy::prelude::*;

use crate::{ai::SimpleMotion, combat::status_effects::components::SlowedStatus};

pub fn on_slow_applied(
    trigger: Trigger<OnInsert, SlowedStatus>,
    status_query: Query<(&ChildOf, &SlowedStatus)>,
    mut motion_query: Query<&mut SimpleMotion>,
) {
    let Ok((child_of, slowed)) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(child_of.parent) {
        motion.slow(slowed.slow_percentage);
    }
}

pub fn on_slow_removed(
    trigger: Trigger<OnRemove, SlowedStatus>,
    status_query: Query<&ChildOf, With<SlowedStatus>>,
    mut motion_query: Query<&mut SimpleMotion>,
) {
    let Ok(child_of) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(child_of.parent) {
        motion.remove_debuff();
    }
}
