use bevy::prelude::*;

use crate::combat::{
    Health,
    damage::{AttemptDamage, Damage},
    status_effects::{Status, StatusApplied, StatusOf},
};

#[derive(Component, Clone)]
#[require(Status)]
pub struct Burning {
    pub damage: f32,
    pub damage_frequency: Timer,
}

impl Default for Burning {
    fn default() -> Burning {
        Burning {
            damage: 2.0,
            damage_frequency: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

const RED_COLOR: bevy::prelude::Color = Color::srgb(1.0, 0.0, 0.0);

pub fn tick_burn(mut burn_query: Query<&mut Burning>, time: Res<Time>) {
    for mut burn_status in burn_query.iter_mut() {
        burn_status.damage_frequency.tick(time.delta());
    }
}

// TODO: Modify this to be a "DamagePerSecond" component + system since it isn't specific to burning
pub fn while_burning(
    mut commands: Commands,
    status_query: Query<(&Burning, &StatusOf)>,
    mut health_query: Query<Entity, With<Health>>,
) {
    for (burn, status_of) in status_query.iter() {
        if let Ok(entity) = health_query.get_mut(status_of.0)
            && burn.damage_frequency.just_finished()
        {
            commands.trigger(AttemptDamage {
                entity,
                ignore_invulnerable: true,
                damage_source: None,
                damage: Damage::Single(burn.damage),
            });
        }
    }
}

pub fn apply_burning(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Burning>, Without<StatusApplied>)>,
    mut sprite_query: Query<&mut Sprite>,
) {
    status_query.iter().for_each(|(status, status_of)| {
        commands.entity(status).insert(StatusApplied);

        if let Ok(mut affected_sprite) = sprite_query.get_mut(status_of.0) {
            affected_sprite.color = RED_COLOR;
        }
    });
}

pub fn on_burn_removed(
    burning_status: On<Remove, Burning>,
    status_query: Query<&StatusOf, With<Burning>>,
    mut sprite_query: Query<&mut Sprite>,
) {
    let Ok(status_of) = status_query.get(burning_status.entity) else {
        return;
    };

    if let Ok(mut burnt_sprite) = sprite_query.get_mut(status_of.0) {
        burnt_sprite.color = Color::default();
    }
}
