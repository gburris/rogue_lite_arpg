use bevy::prelude::*;

use crate::{combat::status_effects::components::SlowedStatus, movement::components::SimpleMotion};

pub fn on_slow_applied(
    trigger: Trigger<OnInsert, SlowedStatus>,
    status_query: Query<(&Parent, &SlowedStatus)>,
    mut parent_speed_query: Query<&mut SimpleMotion>,
) {
    let Ok((parent, slowed)) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = parent_speed_query.get_mut(parent.get()) {
        motion.slow(slowed.slow_percentage);
    }
}

pub fn on_slow_removed(
    trigger: Trigger<OnRemove, SlowedStatus>,
    status_query: Query<&Parent, With<SlowedStatus>>,
    mut parent_speed_query: Query<&mut SimpleMotion>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = parent_speed_query.get_mut(parent.get()) {
        motion.remove_debuff();
    }
}
