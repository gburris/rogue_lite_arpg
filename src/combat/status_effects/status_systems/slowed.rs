use bevy::prelude::*;

use crate::{ai::SimpleMotion, combat::status_effects::components::SlowedStatus};

pub fn on_slow_applied(
    trigger: Trigger<OnInsert, SlowedStatus>,
    status_query: Query<(&ChildOf, &SlowedStatus)>,
    mut ChildOf_speed_query: Query<&mut SimpleMotion>,
) {
    let Ok((ChildOf, slowed)) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = ChildOf_speed_query.get_mut(ChildOf.get()) {
        motion.slow(slowed.slow_percentage);
    }
}

pub fn on_slow_removed(
    trigger: Trigger<OnRemove, SlowedStatus>,
    status_query: Query<&ChildOf, With<SlowedStatus>>,
    mut ChildOf_speed_query: Query<&mut SimpleMotion>,
) {
    let Ok(ChildOf) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = ChildOf_speed_query.get_mut(ChildOf.get()) {
        motion.remove_debuff();
    }
}
