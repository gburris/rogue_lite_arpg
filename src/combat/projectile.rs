use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
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
    pub spawn_offset: f32,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            damage: Damage::Range((5.0, 10.0)),
            speed: 600.0,
            spawn_offset: 25.0,
        }
    }
}

#[derive(Component)]
#[relationship(relationship_target = Projectiles)]
pub struct ProjectileOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ProjectileOf)]
pub struct Projectiles(Vec<Entity>);

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

pub fn fireball(sprites: &SpriteAssets, texture_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Projectile::default(),
        // EffectsList {
        //     effects: vec![ApplyStatus {
        //         status: StatusType::Burning(BurningStatus::default()),
        //         duration: 2.0,
        //     }],
        // },
        Sprite::from_atlas_image(
            sprites.fire_ball.clone(),
            TextureAtlas {
                layout: texture_layouts.fireball_layout.clone(),
                index: 0,
            },
        ),
    )
}

pub fn icicle(sprites: &SpriteAssets, texture_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Projectile {
            damage: Damage::Range((12.0, 25.0)),
            speed: 500.0,
            spawn_offset: 25.0,
        },
        EffectsList {
            effects: vec![ApplyStatus {
                status: StatusType::Frozen,
                duration: 2.0,
            }],
        },
        Sprite::from_atlas_image(
            sprites.ice_bolt.clone(),
            TextureAtlas {
                layout: texture_layouts.ice_bolt_layout.clone(),
                index: 0,
            },
        ),
    )
}
