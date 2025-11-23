use avian2d::prelude::*;
use bevy::{color::palettes::tailwind::YELLOW_300, ecs::entity_disabling::Disabled, prelude::*};
use bevy_lit::prelude::PointLight2d;

use crate::{
    combat::damage::{AttemptDamage, Damage, HurtBox, Knockback},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, handle_collisions.in_set(InGameSystems::Collision))
        .add_observer(on_fire_projectile);

    app.add_observer(despawn_all::<CleanupZone, Projectile>);
}

#[derive(Component, Clone)]
#[require(
    Lifespan::new(1.0),
    Sensor,
    RigidBody::Kinematic,
    CollidingEntities,
    AnimationIndices::Cycle((0..=4).cycle()),
    AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    Disabled
)]
pub struct Projectile {
    pub damage: Damage,
    pub speed: f32,
    angle_offset: f32,
    spawn_offset: f32,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            damage: Damage::Range((5.0, 10.0)),
            speed: 600.0,
            angle_offset: 0.0,
            spawn_offset: 0.0,
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
            speed: 450.0,
            spawn_offset: 20.0,
            angle_offset,
        },
        Sprite::from_atlas_image(
            sprites.fire_ball.clone(),
            TextureAtlas {
                layout: sprite_layouts.fireball_layout.clone(),
                index: 0,
            },
        ),
        PointLight2d {
            color: Color::from(YELLOW_300),
            intensity: 1.4,
            falloff: 5.0,
            outer_radius: 30.0,
            ..default()
        },
        Collider::circle(10.0),
        AnimationTimer(Timer::from_seconds(0.042, TimerMode::Repeating)),
        Knockback(5.0),
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
            speed: 350.0,
            spawn_offset: 30.0,
            angle_offset,
        },
        Sprite::from_atlas_image(
            sprites.ice_bolt.clone(),
            TextureAtlas {
                layout: sprite_layouts.ice_bolt_layout.clone(),
                index: 0,
            },
        ),
        Knockback(5.0),
        related!(Effects[(Frozen, Lifespan::new(0.7))]),
    )
}

fn handle_collisions(
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &LinearVelocity, &CollidingEntities, Entity)>,
    hurt_box_query: Query<&HurtBox>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, velocity, colliding_entities, projectile_entity) in projectile_query.iter() {
        // ignore further collisions after ANY collision with the projectile
        if let Some(&colliding_entity) = colliding_entities.iter().next() {
            // If the thing we collide with has a HurtBox, lets try to damage it!
            if hurt_box_query.contains(colliding_entity) {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    damage: projectile.damage,
                    damage_source: Some(projectile_entity),
                    direction: Some(velocity.normalize()),
                    ..default()
                });
            }
            if reflector_query.contains(colliding_entity) {
                continue;
            }
            commands.entity(projectile_entity).despawn();
        }
    }
}

#[derive(EntityEvent)]
pub struct FireProjectile {
    #[event_target]
    projectile: Entity,
    damage_source: DamageSource,
    position: Vec2,
    aim_direction: Vec2,
}

impl From<(Entity, DamageSource, Vec2, Vec2)> for FireProjectile {
    fn from(
        (projectile, damage_source, position, aim_direction): (Entity, DamageSource, Vec2, Vec2),
    ) -> Self {
        FireProjectile {
            projectile,
            damage_source,
            position,
            aim_direction: aim_direction.normalize(),
        }
    }
}

fn on_fire_projectile(
    fire: On<FireProjectile>,
    mut commands: Commands,
    projectile_query: Query<&Projectile, Allow<Disabled>>,
) {
    let projectile = projectile_query.get(fire.projectile).unwrap();

    // Rotate the aim direction by the projectile’s angle offset
    let projectile_direction = fire
        .aim_direction
        .rotate(Vec2::from_angle(projectile.angle_offset));

    let position = fire.position + (projectile_direction * projectile.spawn_offset);

    commands
        .entity(fire.projectile)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.linked_cloning(true);
        })
        .remove::<(ProjectileOf, Disabled)>()
        .insert((
            Position(position),
            Rotation::radians(projectile_direction.to_angle()),
            Transform {
                translation: position.extend(ZLayer::InAir.z()),
                rotation: Quat::from_rotation_z(projectile_direction.to_angle()),
                ..default()
            },
            LinearVelocity(projectile_direction * projectile.speed),
            CollisionLayers::new(
                GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                LayerMask::from(fire.damage_source) | GameCollisionLayer::HighObstacle,
            ),
        ));
}
