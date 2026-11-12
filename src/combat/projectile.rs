use std::collections::HashSet;

use avian2d::prelude::*;
use bevy::color::palettes::tailwind::{BLUE_400, YELLOW_300};
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

use super::damage::{AttemptDamage, Damage, HurtBox, Knockback};
use super::status_effects::prelude::Burning;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        damage_on_projectile_hit.in_set(InGameSystems::Collision),
    )
    .add_observer(on_fire_projectile);
}

#[derive(Component, Clone)]
#[require(
    Lifespan::new(1.0),
    Sensor,
    RigidBody::Kinematic,
    CollidingEntities,
    AnimationIndices::Cycle((0..=4).cycle()),
    AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    Disabled,
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
            damage: Damage::Range {
                min: 5.0,
                max: 10.0,
            },
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

/// Travel speed of a standard fireball, in world units per second.
pub const FIREBALL_SPEED: f32 = 450.0;
/// Distance a standard fireball covers during its one-second default lifespan.
pub const FIREBALL_RANGE: f32 = 450.0;

pub fn fireball(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    angle_offset: f32,
) -> impl Scene {
    fireball_with_speed(sprites, sprite_layouts, angle_offset, FIREBALL_SPEED)
}

/// Builds a fireball at a custom speed, scaling its lifespan so it still covers
/// the standard fireball's travel distance.
pub fn fireball_with_speed(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    angle_offset: f32,
    speed: f32,
) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.fire_ball.clone(),
        TextureAtlas {
            layout: sprite_layouts.fireball.clone(),
            index: 0,
        },
    );
    bsn! {
        Projectile {
            damage: Damage::Single(3.0),
            speed,
            spawn_offset: 20.0,
            angle_offset,
        }
        Lifespan::new({FIREBALL_RANGE / speed})
        template(move |_| Ok(sprite.clone()))
        @point_light(Color::from(YELLOW_300), 1.4, 50.0, 5.0)
        ~Collider::circle(10.0)
        AnimationTimer(Timer::from_seconds(0.042, TimerMode::Repeating))
        Knockback(5.0)
        Effects [
            Burning
            Lifespan::new(2.5)
        ]
    }
}

pub fn icebolt(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    angle_offset: f32,
) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.ice_bolt.clone(),
        TextureAtlas {
            layout: sprite_layouts.ice_bolt.clone(),
            index: 0,
        },
    );
    bsn! {
        Projectile {
            damage: Damage::Range {
                min: 10.0,
                max: 20.0,
            },
            speed: 350.0,
            spawn_offset: 26.0,
            angle_offset,
        }
        template(move |_| Ok(sprite.clone()))
        @point_light(Color::from(BLUE_400), 1.4, 80.0, 5.0)
        ~Collider::rectangle(40.0, 5.0)
        ~{AnimationIndices::Cycle((0..=2).cycle())}
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
        Knockback(5.0)
        Effects [
            Frozen
            Lifespan::new(0.7)
        ]
    }
}

fn damage_on_projectile_hit(
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &LinearVelocity, &CollidingEntities, Entity)>,
    attacker_query: Query<&ProjectileAttacker>,
    hurt_box_query: Query<&HurtBox>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, velocity, colliding_entities, projectile_entity) in &projectile_query {
        // ignore further collisions after ANY collision with the projectile
        if let Some(&colliding_entity) = colliding_entities.iter().next() {
            // If the thing we collide with has a HurtBox, lets try to damage it!
            if hurt_box_query.contains(colliding_entity) {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    damage: projectile.damage,
                    source_item: Some(projectile_entity),
                    attacker: attacker_query
                        .get(projectile_entity)
                        .ok()
                        .and_then(|attacker| attacker.attacker),
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

/// Character that fired a projectile, stamped when it is fired and transferred
/// to the blocker on reflection.
#[derive(Component, Clone, Copy)]
pub struct ProjectileAttacker {
    pub attacker: Option<Entity>,
}

/// Request to launch a projectile template. The template is cloned; the clone
/// becomes the live projectile carrying `ProjectileAttacker`.
#[derive(EntityEvent)]
pub struct FireProjectile {
    #[event_target]
    pub item: Entity,
    pub side: CombatSide,
    pub holder: Option<Entity>,
    pub position: Vec2,
    pub aim_direction: Vec2,
}

fn on_fire_projectile(
    fire: On<FireProjectile>,
    mut commands: Commands,
    projectile_query: Query<&Projectile, Allow<Disabled>>,
) {
    let Ok(projectile) = projectile_query.get(fire.item) else {
        warn!(
            "Tried to fire projectile from missing template {:?}",
            fire.item
        );
        return;
    };
    let aim_direction = fire.aim_direction.normalize();

    // Rotate the aim direction by the projectile’s angle offset
    let projectile_direction = aim_direction.rotate(Vec2::from_angle(projectile.angle_offset));

    let position = fire.position + (projectile_direction * projectile.spawn_offset);

    let mut template = commands.entity(fire.item);
    let mut new_projectile = template.clone_and_spawn_with_opt_out(|builder| {
        builder.linked_cloning(true);
    });

    let projectile_entity = new_projectile.id();

    new_projectile.remove::<(ProjectileOf, Disabled)>().insert((
        ColliderOf {
            body: projectile_entity,
        },
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
            LayerMask::from(fire.side.target_hurtbox()) | GameCollisionLayer::HighObstacle,
        ),
        ProjectileAttacker {
            attacker: fire.holder,
        },
        DespawnOnExit(AppState::Playing),
    ));
}

/// Sends projectiles back at its holder's enemies on contact instead of being destroyed by them.
/// It reflects only while it has a collider.
#[derive(Component, Default, Clone)]
#[require(CollidingEntities, Sensor)]
pub struct ProjectileReflection {
    reflected: HashSet<Entity>,
}

/// Reverses each projectile touching a reflector, at most once per projectile, and hands it to
/// the reflector's holder.
pub(super) fn reflect_projectiles(
    mut commands: Commands,
    mut reflector_query: Query<
        (&mut ProjectileReflection, &CollidingEntities, &ChildOf),
        With<Collider>,
    >,
    mut projectile_query: Query<(&mut LinearVelocity, &mut Transform), With<Projectile>>,
    holder_query: Query<&CombatSide>,
) {
    for (mut reflection, colliding_entities, child_of) in &mut reflector_query {
        reflection
            .reflected
            .retain(|&projectile| projectile_query.contains(projectile));
        for &colliding_entity in colliding_entities.iter() {
            if reflection.reflected.contains(&colliding_entity) {
                continue;
            }
            if let Ok((mut linear_velocity, mut transform)) =
                projectile_query.get_mut(colliding_entity)
            {
                // The blocker takes over: the reflected projectile now hurts the blocker's enemies.
                let Ok(side) = holder_query.get(child_of.parent()) else {
                    continue;
                };

                // Reverse direction of projectile! Reflect!
                linear_velocity.0 = -linear_velocity.0;

                // Rotate projectile sprite to face new velocity direction
                transform.rotation = Quat::from_rotation_z(linear_velocity.0.to_angle());

                commands.entity(colliding_entity).insert((
                    CollisionLayers::new(
                        GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                        LayerMask::from(side.target_hurtbox()) | GameCollisionLayer::HighObstacle,
                    ),
                    ProjectileAttacker {
                        attacker: Some(child_of.parent()),
                    },
                ));

                reflection.reflected.insert(colliding_entity);
            }
        }
    }
}
