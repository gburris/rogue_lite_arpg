use bevy::prelude::*;

use crate::{
    combat::status_effects::{
        components::{FrozenStatus, StatusType},
        events::ApplyStatus,
    },
    despawn::components::LiveDuration,
};

const BLUE_COLOR: bevy::prelude::Color = Color::srgb(0.0, 0.0, 1.0);

pub fn on_frozen_applied(
    trigger: Trigger<OnInsert, FrozenStatus>,
    mut commands: Commands,
    status_query: Query<(&Parent, &LiveDuration), With<FrozenStatus>>,
    mut parent_sprite: Query<&mut Sprite>,
) {
    let Ok((parent, duration)) = status_query.get(trigger.entity()) else {
        return;
    };

    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Stunned,
            duration: duration.0.remaining_secs(), // make sure stun lasts while frozen
        },
        parent.get(),
    );

    if let Ok(mut parent_sprite) = parent_sprite.get_mut(parent.get()) {
        parent_sprite.color = BLUE_COLOR;
    }
}

pub fn on_frozen_removed(
    trigger: Trigger<OnRemove, FrozenStatus>,
    status_query: Query<&Parent, With<FrozenStatus>>,
    mut parent_sprite: Query<&mut Sprite>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut parent_sprite) = parent_sprite.get_mut(parent.get()) {
        parent_sprite.color = Color::default();
    }
}
