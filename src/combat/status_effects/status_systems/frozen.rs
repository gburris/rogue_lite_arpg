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
    status_query: Query<(&ChildOf, &LiveDuration), With<FrozenStatus>>,
    mut ChildOf_sprite: Query<&mut Sprite>,
) {
    let Ok((ChildOf, duration)) = status_query.get(trigger.target()) else {
        return;
    };

    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Stunned,
            duration: duration.0.remaining_secs(), // make sure stun lasts while frozen
        },
        ChildOf.get(),
    );

    if let Ok(mut ChildOf_sprite) = ChildOf_sprite.get_mut(ChildOf.get()) {
        ChildOf_sprite.color = BLUE_COLOR;
    }
}

pub fn on_frozen_removed(
    trigger: Trigger<OnRemove, FrozenStatus>,
    status_query: Query<&ChildOf, With<FrozenStatus>>,
    mut ChildOf_sprite: Query<&mut Sprite>,
) {
    let Ok(ChildOf) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut ChildOf_sprite) = ChildOf_sprite.get_mut(ChildOf.get()) {
        ChildOf_sprite.color = Color::default();
    }
}
