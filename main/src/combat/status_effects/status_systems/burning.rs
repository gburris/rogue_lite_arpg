use bevy::prelude::*;

use crate::combat::{
    damage::{AttemptDamageEvent, Damage},
    status_effects::components::BurningStatus,
    Health,
};

const RED_COLOR: bevy::prelude::Color = Color::srgb(1.0, 0.0, 0.0);

pub fn tick_burn(mut burn_query: Query<&mut BurningStatus>, time: Res<Time>) {
    for mut burn_status in burn_query.iter_mut() {
        burn_status.damage_frequency.tick(time.delta());
    }
}

// TODO: Modify this to be a "DamagePerSecond" component + system since it isn't specific to burning
pub fn while_burning(
    status_query: Query<(&BurningStatus, &Parent)>,
    mut commands: Commands,
    mut parent_query: Query<Entity, With<Health>>,
) {
    for (burn, parent) in status_query.iter() {
        if let Ok(entity) = parent_query.get_mut(parent.get()) {
            if burn.damage_frequency.just_finished() {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage_source: None,
                        damage: Damage::Single(burn.damage),
                    },
                    entity,
                );
            }
        }
    }
}

pub fn on_burn_applied(
    trigger: Trigger<OnInsert, BurningStatus>,
    status_query: Query<&Parent, With<BurningStatus>>,
    mut parent_sprite: Query<&mut Sprite>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut parent_sprite) = parent_sprite.get_mut(parent.get()) {
        parent_sprite.color = RED_COLOR;
    }
}

pub fn on_burn_removed(
    trigger: Trigger<OnRemove, BurningStatus>,
    status_query: Query<&Parent, With<BurningStatus>>,
    mut parent_sprite: Query<&mut Sprite>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    if let Ok(mut parent_sprite) = parent_sprite.get_mut(parent.get()) {
        parent_sprite.color = Color::default();
    }
}
