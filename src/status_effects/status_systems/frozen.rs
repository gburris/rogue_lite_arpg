use bevy::prelude::*;

use crate::{
    despawn::components::Duration,
    resources::assets::SpriteAssets,
    status_effects::{
        components::{FrozenStatus, StatusType},
        events::ApplyStatus,
    },
};

pub fn on_frozen_applied(
    trigger: Trigger<OnInsert, FrozenStatus>,
    mut commands: Commands,
    status_query: Query<(&Parent, &Duration), With<FrozenStatus>>,
    sprites: Res<SpriteAssets>,
) {
    let Ok((parent, duration)) = status_query.get(trigger.entity()) else {
        return;
    };

    info!("Enemy frozen, applying stun");
    commands.trigger_targets(
        ApplyStatus {
            status: StatusType::Stunned,
            duration: duration.0.remaining_secs(), // make sure stun lasts while frozen
        },
        parent.get(),
    );

    commands
        .entity(parent.get())
        .insert(Sprite::from_image(sprites.merman_freezing.clone()));
}

pub fn on_frozen_removed(
    trigger: Trigger<OnRemove, FrozenStatus>,
    mut commands: Commands,
    status_query: Query<&Parent, With<FrozenStatus>>,
    sprites: Res<SpriteAssets>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    commands
        .entity(parent.get())
        .insert(Sprite::from_image(sprites.merman_enemy.clone()));
}
