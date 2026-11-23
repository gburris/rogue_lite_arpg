use avian2d::prelude::*;
use bevy::color::palettes::tailwind::{BLUE_400, YELLOW_300};
use bevy::ecs::entity_disabling::Disabled;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use super::damage::{AttemptDamage, CreditedAttackOf, Damage, Knockback};
use super::hitbox::HurtBox;
use super::status_effects::Burning;
use crate::interval::Interval;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        damage_on_projectile_hit.in_set(InGameSystems::Collision),
    )
    .add_observer(on_fire_projectiles);
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
                amount: Interval::new(5.0, 10.0),
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
const FIREBALL_RANGE: f32 = 450.0;

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
    bsn! {
        @atlas_sprite(sprites.fire_ball.clone(), sprite_layouts.fireball.clone())
        Projectile {
            damage: Damage::Single(3.0),
            speed,
            spawn_offset: 20.0,
            angle_offset,
        }
        Lifespan::new({FIREBALL_RANGE / speed})
        @point_light(Color::from(YELLOW_300), 1.4, 50.0, 5.0)
        ~Collider::circle(10.0)
        AnimationTimer(Timer::from_seconds(0.042, TimerMode::Repeating))
        Knockback(1250.0)
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
    bsn! {
        @atlas_sprite(sprites.ice_bolt.clone(), sprite_layouts.ice_bolt.clone())
        Projectile {
            damage: Damage::Range {
                amount: Interval::new(10.0, 20.0),
            },
            speed: 350.0,
            spawn_offset: 26.0,
            angle_offset,
        }
        @point_light(Color::from(BLUE_400), 1.4, 80.0, 5.0)
        ~Collider::rectangle(40.0, 5.0)
        ~{AnimationIndices::Cycle((0..=2).cycle())}
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
        Knockback(1250.0)
        Effects [
            Frozen
            Lifespan::new(0.7)
            --
            @stat_modifier(Stat::Speed, -1.0)
            Lifespan::new(0.7)
        ]
    }
}

fn damage_on_projectile_hit(
    mut commands: Commands,
    projectile_query: Query<(
        &Projectile,
        &LinearVelocity,
        &CollidingEntities,
        Entity,
        Option<&CreditedAttackOf>,
    )>,
    hurt_box_query: Query<&HurtBox>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, velocity, colliding_entities, projectile_entity, credited_attack_of) in
        &projectile_query
    {
        // ignore further collisions after ANY collision with the projectile
        if let Some(&colliding_entity) = colliding_entities.iter().next() {
            // Damage the touched entity if it has a hurtbox.
            if hurt_box_query.contains(colliding_entity) {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    damage: projectile.damage,
                    source_item: Some(projectile_entity),
                    attacker: credited_attack_of
                        .map(|credited_attack_of| credited_attack_of.attacker),
                    direction: velocity.try_normalize(),
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

/// Gap a launch point keeps from a wall, about a projectile's radius.
const WALL_CLEARANCE: f32 = 12.0;

/// Request to launch every projectile template of `source` from `position` along
/// `aim_direction`, on behalf of `holder`. Each template is cloned; the clone becomes the live
/// projectile, credited to `holder` with [`CreditedAttackOf`].
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct FireProjectiles {
    #[event_target]
    pub source: Entity,
    pub holder: Entity,
    pub position: Vec2,
    pub aim_direction: Vec2,
}

/// `launch_point` pulled back toward the holder's `feet` until it is [`WALL_CLEARANCE`] short of
/// any wall in between. A launch point held up against a wall sits inside it, where a projectile
/// would be destroyed the moment it spawns, whichever way it was aimed.
pub fn pull_back_from_walls(spatial_query: &SpatialQuery, feet: Vec2, launch_point: Vec2) -> Vec2 {
    let feet_to_launch = launch_point - feet;
    let Ok(direction) = Dir2::new(feet_to_launch) else {
        return launch_point;
    };
    let walls = SpatialQueryFilter::from_mask(GameCollisionLayer::HighObstacle);

    match spatial_query.cast_ray(feet, direction, feet_to_launch.length(), true, &walls) {
        Some(wall_hit) => feet + direction * (wall_hit.distance - WALL_CLEARANCE).max(0.0),
        None => launch_point,
    }
}

fn on_fire_projectiles(
    fire: On<FireProjectiles>,
    mut commands: Commands,
    sources: Query<&Projectiles>,
    templates: Query<&Projectile, Allow<Disabled>>,
    holders: Query<(&CombatSide, &StatMultipliers)>,
) -> Result {
    let projectiles = sources.get(fire.source)?;
    let Ok((&side, stat_multipliers)) = holders.get(fire.holder) else {
        return Ok(());
    };
    let Some(aim_direction) = fire.aim_direction.try_normalize() else {
        return Ok(());
    };
    for template in projectiles.iter() {
        let projectile = templates.get(template)?;
        let projectile_direction = Rot2::radians(projectile.angle_offset) * aim_direction;
        let projectile_angle = projectile_direction.to_angle();
        let position = fire.position + (projectile_direction * projectile.spawn_offset);

        let mut template_commands = commands.entity(template);
        let mut new_projectile = template_commands.clone_and_spawn_with_opt_out(|builder| {
            builder.linked_cloning(true);
        });
        let projectile_entity = new_projectile.id();
        new_projectile.insert((
            ColliderOf {
                body: projectile_entity,
            },
            Position(position),
            Rotation::radians(projectile_angle),
            Transform {
                translation: position.extend(ZLayer::InAir.z()),
                rotation: Quat::from_rotation_z(projectile_angle),
                ..default()
            },
            LinearVelocity(projectile_direction * projectile.speed),
            // The holder's damage multiplier is locked in at launch.
            Projectile {
                damage: projectile.damage.scaled(stat_multipliers.damage),
                ..projectile.clone()
            },
            side.projectile_layers(),
            CreditedAttackOf {
                attacker: fire.holder,
            },
            DespawnOnExit(AppState::Playing),
        ));
        // Removed after the insert, since inserting `Projectile` re-adds its required `Disabled`.
        new_projectile.remove::<(ProjectileOf, Disabled)>();
    }
    Ok(())
}

/// Sends projectiles back at its holder's enemies on contact instead of being destroyed by them.
/// On an item it only marks the item as reflecting; a part of each use (see [`ActionUsePartOf`])
/// gets its own copy, which does the reflecting.
#[derive(Component, Default, Clone)]
#[require(CollidingEntities, Sensor)]
pub struct ProjectileReflection {
    reflected: HashSet<Entity>,
}

/// Reverses each projectile touching a reflector, at most once per projectile, and assigns it to
/// the reflector's holder.
pub(super) fn reflect_projectiles(
    mut commands: Commands,
    mut reflector_query: Query<(
        &mut ProjectileReflection,
        &CollidingEntities,
        &ActionUsePartOf,
    )>,
    action_uses: Query<&ActionInUseOf>,
    mut projectile_query: Query<
        (&mut LinearVelocity, &mut Transform, Option<&mut Lifespan>),
        With<Projectile>,
    >,
    holder_query: Query<&CombatSide>,
) {
    for (mut reflection, colliding_entities, action_use_part_of) in &mut reflector_query {
        let Ok(&ActionInUseOf { holder }) = action_uses.get(action_use_part_of.action_use) else {
            continue;
        };

        reflection
            .reflected
            .retain(|&projectile| projectile_query.contains(projectile));
        for &colliding_entity in colliding_entities.iter() {
            if reflection.reflected.contains(&colliding_entity) {
                continue;
            }
            if let Ok((mut linear_velocity, mut transform, lifespan)) =
                projectile_query.get_mut(colliding_entity)
            {
                // The reflected projectile now damages the blocker's enemies.
                let Ok(side) = holder_query.get(holder) else {
                    continue;
                };

                linear_velocity.0 = -linear_velocity.0;

                // Rotate the sprite to face the new velocity.
                transform.rotation = Quat::from_rotation_z(linear_velocity.0.to_angle());

                // Reset its range so it can travel back to the shooter.
                if let Some(mut lifespan) = lifespan {
                    lifespan.reset();
                }

                commands.entity(colliding_entity).insert((
                    side.projectile_layers(),
                    CreditedAttackOf { attacker: holder },
                ));

                reflection.reflected.insert(colliding_entity);
            }
        }
    }
}
