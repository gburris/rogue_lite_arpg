use bevy::prelude::*;

use crate::{
    combat::status_effects::components::{FrozenStatus, StatusType},
    utility::Lifespan,
};

use super::{
    components::StunnedStatus,
    events::{ApplyEffect, ApplyStatus},
};

/**
 * Distributes the list of statuses as status events
 */
pub fn on_effect_apply(trigger: Trigger<ApplyEffect>, mut commands: Commands) {
    for status in trigger.effect.clone() {
        commands.trigger_targets(status, trigger.target());
    }
}

pub fn on_status_apply(trigger: Trigger<ApplyStatus>, mut commands: Commands) {
    let child = commands.spawn_empty().id();

    // Attach status to ChildOf
    commands.entity(trigger.target()).add_child(child);

    let mut child_commands = commands.entity(child);

    // Every status must have a duration
    child_commands.insert(Lifespan::new(trigger.event().duration));

    let _ = match &trigger.event().status {
        StatusType::Burning(component) => child_commands.insert(component.clone()),
        StatusType::Slowed(component) => child_commands.insert(component.clone()),
        StatusType::Stunned => child_commands.insert(StunnedStatus),
        StatusType::Frozen => child_commands.insert(FrozenStatus),
    };
}
