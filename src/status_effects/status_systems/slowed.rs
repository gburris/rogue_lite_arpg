use bevy::prelude::*;

use crate::{
    enemy::Enemy, movement::components::SimpleMotion, status_effects::components::SlowedStatus,
};

pub fn on_slow_applied(
    trigger: Trigger<OnInsert, SlowedStatus>,
    status_query: Query<(&Parent, &SlowedStatus)>,
    mut parent_speed_query: Query<&mut SimpleMotion, With<Enemy>>,
) {
    let Ok((parent, slowed)) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = parent_speed_query.get_mut(parent.get()) {
        motion.current_speed = motion.max_speed * slowed.slow_percentage;
    }
}

pub fn on_slow_removed(
    trigger: Trigger<OnRemove, SlowedStatus>,
    status_query: Query<&Parent, With<SlowedStatus>>,
    mut parent_speed_query: Query<&mut SimpleMotion, With<Enemy>>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = parent_speed_query.get_mut(parent.get()) {
        motion.current_speed = motion.max_speed;
    }
}
