use bevy::prelude::*;

use crate::{
    combat::status_effects::{
        components::{SlowedStatus, StatusType, StunnedStatus},
        events::ApplyStatus,
    },
    enemy::Enemy,
    movement::components::SimpleMotion,
};

pub fn on_stun_applied(
    trigger: Trigger<OnInsert, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut motion_query: Query<&mut SimpleMotion, With<Enemy>>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(parent.get()) {
        motion.can_move = false;
    }
}

pub fn on_stun_removed(
    trigger: Trigger<OnRemove, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut motion_query: Query<&mut SimpleMotion, With<Enemy>>,
    mut commands: Commands,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(parent.get()) {
        motion.can_move = true;
    }

    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Slowed(SlowedStatus::default()),
            duration: 3.0,
        },
        parent.get(),
    );
}
