use crate::{
    components::{
        animation_indices::AnimationIndices, damage_effect::DamageEffect, projectile::Projectile,
        AnimationTimer, BurningEffect, FreezingEffect,
    },
    helpers::labels::GameCollisionLayer,
    resources::assets::SpriteAssets,
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
        sprites: &Res<SpriteAssets>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) {
        let spell_speed = 300.0;
        let direction = Mat3::from_quat(caster_transform.rotation).x_axis;
        let velocity = (direction * spell_speed).truncate();

        match spell_type {
            SpellType::Fireball => {
                commands.spawn((
                    Projectile::new(300.0),
                    crate::spells::components::Fireball,
                    caster_transform,
                    DamageEffect { base_damage: 10.0 },
                    LinearVelocity(velocity),
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
                    Sprite::from_image(sprites.fire_bolt.clone()),
                ));
            }
            SpellType::Icebolt => {
                let animation_indices = AnimationIndices { first: 0, last: 4 };
                let texture = sprites.ice_bolt.clone();
                let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);

                commands.spawn((
                    Projectile::new(300.0),
                    crate::spells::components::Icebolt,
                    DamageEffect { base_damage: 8.0 },
                    LinearVelocity(velocity),
                    RigidBody::Dynamic,
                    Collider::rectangle(10.0, 10.0),
                    // Currently projectiles can only collide with enemies
                    CollisionLayers::new(
                        GameCollisionLayer::Projectile,
                        [GameCollisionLayer::Enemy],
                    ),
                    FreezingEffect {
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        slow_percentage: 0.5,
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
