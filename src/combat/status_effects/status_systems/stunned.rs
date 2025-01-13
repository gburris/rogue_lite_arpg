use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    movement::components::IsMoving,
    combat::status_effects::{
        components::{SlowedStatus, StatusType, StunnedStatus},
        events::ApplyStatus,
    },
};

pub fn on_stun_applied(
    trigger: Trigger<OnInsert, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut is_moving_query: Query<&mut IsMoving, With<Enemy>>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut is_moving) = is_moving_query.get_mut(parent.get()) {
        is_moving.0 = false;
    }
}

pub fn on_stun_removed(
    trigger: Trigger<OnRemove, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut is_moving_query: Query<&mut IsMoving, With<Enemy>>,
    mut commands: Commands,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut is_moving) = is_moving_query.get_mut(parent.get()) {
        is_moving.0 = true;
    }

    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Slowed(SlowedStatus::default()),
            duration: 3.0,
        },
        parent.get(),
    );
}
