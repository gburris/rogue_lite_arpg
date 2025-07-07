use std::f32::consts::FRAC_PI_4;

use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::status_effects::components::BurningStatus,
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    utility::Lifespan,
};

use super::{
    damage::{AttemptDamageEvent, Damage, HurtBox},
    shield::components::ProjectileReflection,
    status_effects::{
        components::{EffectsList, StatusType},
        events::ApplyStatus,
    },
};

#[derive(Component, Clone)]
#[require(
    Lifespan::new(1.0),
    Sensor,
    RigidBody,
    Collider::rectangle(10.0, 10.0),
    CollidingEntities,
    AnimationIndices::Cycle((0..=4).cycle()),
    AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    Disabled
)]
pub struct Projectile {
    pub damage: Damage,
    pub speed: f32,
    pub forward_offset: f32,
    pub angle_offset: f32,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            damage: Damage::Range((5.0, 10.0)),
            speed: 600.0,
            forward_offset: 25.0,
            angle_offset: 0.0,
        }
    }
}

#[derive(Component)]
#[relationship(relationship_target = Projectiles)]
pub struct ProjectileOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ProjectileOf)]
pub struct Projectiles(Vec<Entity>);

pub enum BulletSprite {
    Fireball,
    IceBolt,
}

#[derive(Clone)]
pub struct ProjectileBuilder {
    sprite: Sprite,
    damage: Damage,
    speed: f32,
    angle_offset: f32,
}

impl Default for ProjectileBuilder {
    fn default() -> Self {
        Self {
            sprite: Default::default(),
            damage: Damage::Range((5.0, 10.0)),
            speed: 600.0,
            angle_offset: 0.0,
        }
    }
}

impl ProjectileBuilder {
    pub fn new(
        bullet: BulletSprite,
        sprites: &SpriteAssets,
        texture_layouts: &SpriteSheetLayouts,
    ) -> Self {
        let (image, layout) = match bullet {
            BulletSprite::Fireball => (
                sprites.fire_ball.clone(),
                texture_layouts.fireball_layout.clone(),
            ),
            BulletSprite::IceBolt => (
                sprites.ice_bolt.clone(),
                texture_layouts.ice_bolt_layout.clone(),
            ),
        };

        ProjectileBuilder {
            sprite: Sprite::from_atlas_image(
                image,
                TextureAtlas {
                    layout: layout,
                    index: 0,
                },
            ),
            ..default()
        }
    }

    pub fn with_damage(mut self, damage: Damage) -> Self {
        self.damage = damage;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_angle_offset(mut self, offset: f32) -> Self {
        self.angle_offset = offset;
        self
    }

    pub fn build(self) -> impl Bundle {
        (
            Projectile {
                damage: self.damage,
                speed: self.speed,
                forward_offset: 25.0,
                angle_offset: self.angle_offset,
            },
            self.sprite,
        )
    }
}

pub fn handle_collisions(
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &CollidingEntities, Entity)>,
    hurt_box_query: Query<&HurtBox>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, colliding_entities, projectile_entity) in projectile_query.iter() {
        // ignore further collisions after ANY collision with the projectile
        if let Some(&colliding_entity) = colliding_entities.iter().next() {
            // If the thing we collide with has a HurtBox, lets try to damage it!
            if hurt_box_query.contains(colliding_entity) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        ignore_invulnerable: false,
                        damage: projectile.damage,
                        damage_source: Some(projectile_entity),
                    },
                    colliding_entity,
                );
            }
            if reflector_query.contains(colliding_entity) {
                continue;
            }
            commands.entity(projectile_entity).despawn();
        }
    }
}
