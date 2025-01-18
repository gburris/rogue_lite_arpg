use bevy::prelude::*;

use crate::{
    combat::status_effects::{
        components::{AppliedStatus, SlowedStatus, StatusEffect, StatusType, StunnedStatus},
        events::{ApplyEffects, ApplyStatus},
    },
    enemy::Enemy,
    movement::components::IsMoving,
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
    status_query: Query<(Entity, &Parent), (With<StunnedStatus>, With<AppliedStatus>)>,
    mut is_moving_query: Query<&mut IsMoving, With<Enemy>>,
    mut commands: Commands,
) {
    let Ok((status_entity, parent)) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut is_moving) = is_moving_query.get_mut(parent.get()) {
        is_moving.0 = true;
    }

    // To cascade statuses, you can add child to the status and then make
    commands
        .entity(status_entity)
        .with_child((StatusEffect { duration: 3.0 }, SlowedStatus::default()));

    commands.trigger_targets(
        ApplyEffects {
            effect_source: status_entity,
        },
        parent.get(),
    );
}
