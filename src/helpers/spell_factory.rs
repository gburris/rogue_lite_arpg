use crate::components::{
    burning_effect::BurningEffect, collider::Collider, damage_effect::DamageEffect,
    freezing_effect::FreezingEffect, projectile::Projectile,
};
use bevy::prelude::*;
use std::time::Duration;

pub enum SpellType {
    Fireball,
    Icebolt,
}

pub struct SpellFactory;

impl SpellFactory {
    pub fn spawn_spell(
        commands: &mut Commands,
        spell_type: SpellType,
        caster_transform: Transform,
        asset_server: &Res<AssetServer>,
    ) {
        match spell_type {
            SpellType::Fireball => {
                println!("Casting Fireball");
                commands.spawn((
                    Projectile::new(300.0),
                    crate::components::Fireball,
                    caster_transform,
                    DamageEffect { base_damage: 10.0 },
                    Collider {
                        size: Vec2::new(10.0, 10.0),
                    },
                    BurningEffect {
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        tick_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                        damage_per_second: 5.0,
                    },
                    Sprite::from_image(asset_server.load("projectiles/FB001.png")),
                ));
            }
            SpellType::Icebolt => {
                println!("Casting Icebolt");
                commands.spawn((
                    Projectile::new(150.0),
                    crate::components::Icebolt,
                    caster_transform,
                    DamageEffect { base_damage: 8.0 },
                    Collider {
                        size: Vec2::new(10.0, 10.0),
                    },
                    BurningEffect {
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        tick_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                        damage_per_second: 5.0,
                    },
                    Sprite::from_image(asset_server.load("projectiles/frostball.png")),
                ));
            }
        }
    }
}
