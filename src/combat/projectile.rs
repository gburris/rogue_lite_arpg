use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::status_effects::{Burning, Effects, Frozen},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    utility::Lifespan,
};

use super::{
    damage::{AttemptDamageEvent, Damage, HurtBox},
    shield::components::ProjectileReflection,
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

#[derive(Component, Clone)]
#[relationship(relationship_target = Projectiles)]
pub struct ProjectileOf(Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = ProjectileOf, linked_spawn)]
pub struct Projectiles(Vec<Entity>);

pub fn fireball(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    angle_offset: f32,
) -> impl Bundle {
    (
        Projectile {
            damage: Damage::Single(3.0),
            speed: 600.0,
            forward_offset: 25.0,
            angle_offset,
        },
        Sprite::from_atlas_image(
            sprites.fire_ball.clone(),
            TextureAtlas {
                layout: sprite_layouts.fireball_layout.clone(),
                index: 0,
            },
        ),
        related!(Effects[(Burning::default(), Lifespan::new(2.5))]),
    )
}

pub fn icebolt(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    angle_offset: f32,
) -> impl Bundle {
    (
        Projectile {
            damage: Damage::Range((10.0, 20.0)),
            speed: 500.0,
            forward_offset: 25.0,
            angle_offset,
        },
        Sprite::from_atlas_image(
            sprites.ice_bolt.clone(),
            TextureAtlas {
                layout: sprite_layouts.ice_bolt_layout.clone(),
                index: 0,
            },
        ),
        related!(Effects[(Frozen, Lifespan::new(0.7))]),
    )
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
                commands.trigger(AttemptDamageEvent {
                    entity: colliding_entity,
                    ignore_invulnerable: false,
                    damage: projectile.damage,
                    damage_source: Some(projectile_entity),
                });
            }
            if reflector_query.contains(colliding_entity) {
                continue;
            }
            commands.entity(projectile_entity).despawn();
        }
    }
}
