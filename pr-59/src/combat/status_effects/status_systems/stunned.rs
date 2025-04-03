use bevy::prelude::*;

use crate::{
    ai::SimpleMotion,
    combat::status_effects::{
        components::{SlowedStatus, StatusType, StunnedStatus},
        events::ApplyStatus,
    },
};

pub fn on_stun_applied(
    trigger: Trigger<OnInsert, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut motion_query: Query<&mut SimpleMotion>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(parent.get()) {
        motion.stun();
    }
}

pub fn on_stun_removed(
    trigger: Trigger<OnRemove, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut motion_query: Query<&mut SimpleMotion>,
    mut commands: Commands,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(parent.get()) {
        motion.remove_debuff();
    }

    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Slowed(SlowedStatus::default()),
            duration: 3.0,
        },
        parent.get(),
    );
}
