use bevy::prelude::*;

use crate::{
    components::Speed,
    enemy::Enemy,
    status_effects::{
        components::{SlowedStatus, StatusType, StunnedStatus},
        events::ApplyStatus,
    },
};

pub fn on_stun_applied(
    trigger: Trigger<OnInsert, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut parent_speed_query: Query<&mut Speed, With<Enemy>>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut speed) = parent_speed_query.get_mut(parent.get()) {
        info!("Enemy Stunned: {}", trigger.entity());
        speed.velocity = 0.0;
    }
}

pub fn on_stun_removed(
    trigger: Trigger<OnRemove, StunnedStatus>,
    status_query: Query<&Parent, With<StunnedStatus>>,
    mut speed_query: Query<&mut Speed, With<Enemy>>,
    mut commands: Commands,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut speed) = speed_query.get_mut(parent.get()) {
        speed.velocity = speed.max_velocity;
    }

    info!("Enemy stun being removed, applying slow");
    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Slowed(SlowedStatus::default()),
            duration: 3.0,
        },
        parent.get(),
    );
}
