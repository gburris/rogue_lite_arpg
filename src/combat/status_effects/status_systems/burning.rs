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
    status_query: Query<(&BurningStatus, &ChildOf)>,
    mut commands: Commands,
    mut ChildOf_query: Query<Entity, With<Health>>,
) {
    for (burn, ChildOf) in status_query.iter() {
        if let Ok(entity) = ChildOf_query.get_mut(ChildOf.get()) {
            if burn.damage_frequency.just_finished() {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        ignore_invulnerable: true,
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
    status_query: Query<&ChildOf, With<BurningStatus>>,
    mut ChildOf_sprite: Query<&mut Sprite>,
) {
    let Ok(ChildOf) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut ChildOf_sprite) = ChildOf_sprite.get_mut(ChildOf.get()) {
        ChildOf_sprite.color = RED_COLOR;
    }
}

pub fn on_burn_removed(
    trigger: Trigger<OnRemove, BurningStatus>,
    status_query: Query<&ChildOf, With<BurningStatus>>,
    mut ChildOf_sprite: Query<&mut Sprite>,
) {
    let Ok(ChildOf) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut ChildOf_sprite) = ChildOf_sprite.get_mut(ChildOf.get()) {
        ChildOf_sprite.color = Color::default();
    }
}
