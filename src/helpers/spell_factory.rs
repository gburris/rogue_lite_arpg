use crate::{
    components::{
        animation_indices::AnimationIndices, burning_effect::BurningEffect,
        damage_effect::DamageEffect, projectile::Projectile, AnimationTimer,
    },
    helpers::labels::GameCollisionLayer,
};
use avian2d::prelude::*;
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
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) {
        match spell_type {
            SpellType::Fireball => {
                println!("Casting Fireball");
                commands.spawn((
                    Projectile::new(300.0),
                    crate::components::Fireball,
                    caster_transform,
                    DamageEffect { base_damage: 10.0 },
                    RigidBody::Dynamic,
                    Collider::rectangle(10.0, 10.0),
                    // Currently projectiles can only collide with enemies
                    CollisionLayers::new(
                        GameCollisionLayer::Projectile,
                        [GameCollisionLayer::Enemy],
                    ),
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
                let animation_indices = AnimationIndices { first: 0, last: 4 };
                let texture = asset_server.load("projectiles/IceBolt.png");
                let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);

                commands.spawn((
                    Projectile::new(300.0),
                    crate::components::Icebolt,
                    DamageEffect { base_damage: 8.0 },
                    RigidBody::Dynamic,
                    Collider::rectangle(10.0, 10.0),
                    // Currently projectiles can only collide with enemies
                    CollisionLayers::new(
                        GameCollisionLayer::Projectile,
                        [GameCollisionLayer::Enemy],
                    ),
                    BurningEffect {
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        tick_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                        damage_per_second: 5.0,
                    },
                    Sprite::from_atlas_image(
                        texture,
                        TextureAtlas {
                            layout: texture_atlas_layout,
                            index: animation_indices.first,
                        },
                    ),
                    Transform {
                        translation: caster_transform.translation,
                        rotation: caster_transform.rotation,
                        scale: Vec3::splat(2.0),
                    },
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                ));
            }
        }
    }
}
