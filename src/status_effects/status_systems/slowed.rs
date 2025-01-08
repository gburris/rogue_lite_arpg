use bevy::prelude::*;

use crate::{components::Speed, enemy::Enemy, status_effects::components::SlowedStatus};

pub fn on_slow_applied(
    trigger: Trigger<OnInsert, SlowedStatus>,
    status_query: Query<(&Parent, &SlowedStatus)>,
    mut parent_speed_query: Query<&mut Speed, With<Enemy>>,
) {
    let Ok((parent, slowed)) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut speed) = parent_speed_query.get_mut(parent.get()) {
        speed.velocity = speed.velocity * slowed.slow_percentage;
    }
}

pub fn on_slow_removed(
    trigger: Trigger<OnRemove, SlowedStatus>,
    status_query: Query<&Parent, With<SlowedStatus>>,
    mut parent_speed_query: Query<&mut Speed, With<Enemy>>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut speed) = parent_speed_query.get_mut(parent.get()) {
        speed.velocity = speed.max_velocity;
    }
}
