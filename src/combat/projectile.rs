use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer, ZLayer,
    },
    despawn::components::LiveDuration,
};

use super::{
    damage::{AttemptDamageEvent, Damage, DamageSource, HurtBox},
    shield::components::ProjectileReflection,
    status_effects::{
        components::{BurningStatus, EffectsList, StatusType},
        events::ApplyStatus,
    },
};

#[derive(Component, Clone)]
#[require(
    // LiveDuration::new(1.0),
    // Sensor,
    // RigidBody,
    // Collider::rectangle(10.0, 10.0),
    // CollidingEntities,
    // AnimationIndices::Cycle((0..=4).cycle()),
    // AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    Disabled
)]
pub struct Projectile {
    pub damage: Damage,
    pub speed: f32,
}

#[derive(Component)]
#[relationship(relationship_target = Projectiles)]
pub struct ProjectileOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ProjectileOf)]
pub struct Projectiles(Vec<Entity>);

const PROJECTILE_SPAWN_OFFSET: f32 = 25.0;

pub fn spawn(
    damage_source: DamageSource, //Player, enemy, NPC, Party Member
    commands: &mut Commands,
    caster_transform: &Transform,
    caster_aim_position: Vec2,
    projectiles: &Projectiles,
) {
    let caster_direction = caster_transform.local_x().truncate();
    let angle = caster_direction.angle_to(aim_direction);

    // let velocity = aim_direction * projectiles.projectile_speed;

    let starting_positon =
        caster_transform.translation.truncate() + (PROJECTILE_SPAWN_OFFSET * aim_direction);

    // trace!("Spawning projectiles w/ velocity: {}", velocity);

    info!("cloning from relationship");

    for projectile in projectiles.iter() {
        commands.entity(projectile).clone_and_spawn();

        info!("CLONED");

        //cloned.remove::<Disabled>();

        info!("CLONE NOT DISABLED");

        // cloned.insert((
        //     Transform {
        //         translation: starting_positon.extend(ZLayer::InAir.z()),
        //         rotation: Quat::from_rotation_z(angle),
        //         ..default()
        //     },
        //     CollisionLayers::new(
        //         GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
        //         LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
        //     ),
        // ));
    }
}

pub fn on_projectile_spawn(
    trigger: Trigger<OnAdd, Projectile>,
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &Transform)>,
) {
    info!("bruh moment 1");

    if let Ok((projectie, transform)) = projectile_query.get(trigger.target()) {
        commands.entity(trigger.target()).insert(LinearVelocity(
            Vec2::from_angle(transform.rotation.to_axis_angle().1) * projectie.speed,
        ));
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

pub fn fireball(sprites: &SpriteAssets, texture_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Projectile {
            damage: Damage::Range((5.0, 10.0)),
            speed: 600.0,
        },
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
