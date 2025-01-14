use bevy::prelude::*;

use crate::{
    combat::{damage::events::DamageEvent, status_effects::components::BurningStatus},
    configuration::assets::SpriteAssets,
    enemy::Enemy,
};

pub fn tick_burn(mut burn_query: Query<&mut BurningStatus>, time: Res<Time>) {
    for mut burn_status in burn_query.iter_mut() {
        burn_status.damage_frequency.tick(time.delta());
    }
}

pub fn while_burning(
    status_query: Query<(&BurningStatus, &Parent)>,
    mut commands: Commands,
    mut parent_query: Query<Entity, With<Enemy>>,
) {
    for (burn, parent) in status_query.iter() {
        if let Ok(entity) = parent_query.get_mut(parent.get()) {
            if burn.damage_frequency.just_finished() {
                commands.trigger_targets(
                    DamageEvent {
                        damage_source: None,
                        damage: burn.damage,
                        makes_invulnerable: false,
                    },
                    entity,
                );
            }
        }
    }
}

pub fn on_burn_applied(
    trigger: Trigger<OnInsert, BurningStatus>,
    mut commands: Commands,
    status_query: Query<&Parent, With<BurningStatus>>,
    sprites: Res<SpriteAssets>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    commands
        .entity(parent.get())
        .insert(Sprite::from_image(sprites.merman_on_fire.clone()));
}

pub fn on_burn_removed(
    trigger: Trigger<OnRemove, BurningStatus>,
    mut commands: Commands,
    status_query: Query<&Parent, With<BurningStatus>>,
    sprites: Res<SpriteAssets>,
) {
    let Ok(parent) = status_query.get(trigger.entity()) else {
        return;
    };

    commands
        .entity(parent.get())
        .insert(Sprite::from_image(sprites.merman_enemy.clone()));
}
