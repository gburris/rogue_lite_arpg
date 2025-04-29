use bevy::prelude::*;

use crate::{
    combat::status_effects::{
        components::{SlowedStatus, StatusType, StunnedStatus},
        events::ApplyStatus,
    },
    prelude::*,
};

pub fn on_stun_applied(
    trigger: Trigger<OnInsert, StunnedStatus>,
    status_query: Query<&ChildOf, With<StunnedStatus>>,
    mut motion_query: Query<&mut SimpleMotion>,
) {
    let Ok(child_of) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(child_of.parent()) {
        motion.stun();
    }
}

pub fn on_stun_removed(
    trigger: Trigger<OnRemove, StunnedStatus>,
    status_query: Query<&ChildOf, With<StunnedStatus>>,
    mut motion_query: Query<&mut SimpleMotion>,
    mut commands: Commands,
) {
    let Ok(child_of) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(child_of.parent()) {
        motion.remove_debuff();
    }

    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Slowed(SlowedStatus::default()),
            duration: 3.0,
        },
        child_of.parent(),
    );
}
