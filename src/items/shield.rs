use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use super::equipment::prelude::DEFAULT_EQUIPMENT_POSES;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            orient_active_shields.in_set(SimulationSystems::Act),
            reflect_projectiles.in_set(SimulationSystems::Upkeep),
        ),
    );
}

pub fn magic_shield(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Magic Shield"),
        Item::new(355, ItemKind::Tome),
        Equippable::new(EquipmentSlot::Offhand, 0.5, &DEFAULT_EQUIPMENT_POSES),
        ManaCost(5.0),
        ManaDrainRate(20.0),
        ProjectileReflection,
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        Sprite {
            image: sprites.magic_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield.clone(),
                index: 0,
            }),
            ..default()
        },
        observe(on_start_shield),
        observe(on_stop_shield),
    )
}

pub fn knight_shield(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Knight Shield"),
        Item::new(355, ItemKind::Tome),
        Equippable::new(EquipmentSlot::Offhand, 0.5, &DEFAULT_EQUIPMENT_POSES),
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        ManaDrainRate(25.0),
        ManaCost(25.0),
        Sprite {
            image: sprites.knight_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield.clone(),
                index: 0,
            }),
            ..default()
        },
        observe(on_start_shield),
        observe(on_stop_shield),
    )
}

#[derive(Component)]
pub struct Shield {
    pub hitbox: Collider,
}

#[derive(Component, Default)]
#[require(CollidingEntities, Sensor)]
pub struct ProjectileReflection;

impl ProjectileReflection {
    fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HighObstacle, GameCollisionLayer::InAir)
    }
}

#[derive(Component)]
struct ActiveShield {
    projectiles_reflected: HashSet<Entity>,
}

/// Turns each active shield's sprite and offset toward its holder's aim.
fn orient_active_shields(
    mut commands: Commands,
    mut active_shield_query: Query<
        (Entity, &ItemOf, &mut Sprite),
        (With<ActiveShield>, With<Equipped>),
    >,
    holder_query: Query<&Vision>,
) -> Result {
    for (shield_entity, item_of, mut shield_sprite) in &mut active_shield_query {
        let vision = holder_query.get(item_of.holder)?;

        let block_angle = vision.aim_direction.y.atan2(vision.aim_direction.x) + FRAC_PI_2;

        let normalized_angle = if block_angle < -PI {
            block_angle + 2.0 * PI
        } else if block_angle > PI {
            block_angle - 2.0 * PI
        } else {
            block_angle
        };

        let atlas_index = if normalized_angle > -FRAC_PI_4 && normalized_angle < FRAC_PI_4 {
            0
        } else if (-3.0 * FRAC_PI_4..=-FRAC_PI_4).contains(&normalized_angle) {
            2
        } else if (normalized_angle <= -3.0 * FRAC_PI_4) || (normalized_angle >= 3.0 * FRAC_PI_4) {
            3
        } else {
            1
        };

        let offset_distance = 40.0;
        let position_offset = Vec3::new(
            offset_distance * normalized_angle.sin(),
            -offset_distance * normalized_angle.cos(),
            if atlas_index == 0 {
                ZLayer::AboveSprite.z()
            } else {
                ZLayer::BehindSprite.z()
            },
        );

        if let Some(atlas) = &mut shield_sprite.texture_atlas {
            atlas.index = atlas_index;
        }

        commands.entity(shield_entity).insert(Transform::from_xyz(
            position_offset.x,
            position_offset.y,
            position_offset.z,
        ));
    }
    Ok(())
}

/// Start observer: raises the shield and begins blocking.
fn on_start_shield(
    used_shield: On<StartUsingEquipment>,
    mut commands: Commands,
    mut shield_query: Query<&Shield>,
) {
    let Ok(shield) = shield_query.get_mut(used_shield.item) else {
        warn!("Tried to block with invalid shield");
        return;
    };

    commands.entity(used_shield.item).insert((
        EquipmentInUse,
        ActiveShield {
            projectiles_reflected: HashSet::default(),
        },
        shield.hitbox.clone(),
        ProjectileReflection::collision_layers(),
    ));
}

fn reflect_projectiles(
    mut commands: Commands,
    mut shield_query: Query<
        (&mut ActiveShield, &CollidingEntities, &ChildOf),
        With<ProjectileReflection>,
    >,
    mut projectile_query: Query<(&mut LinearVelocity, &mut Transform), With<Projectile>>,
    holder_query: Query<&CombatSide>,
) {
    for (mut shield, colliding_entities, child_of) in &mut shield_query {
        for &colliding_entity in colliding_entities.iter() {
            if shield.projectiles_reflected.contains(&colliding_entity) {
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

                shield.projectiles_reflected.insert(colliding_entity);
            }
        }
    }
}

/// Stop observer: lowers the shield and restores its resting transform.
fn on_stop_shield(
    shield: On<StopUsingEquipment>,
    mut commands: Commands,
    holder_query: Query<&FacingDirection>,
    mut shield_query: Query<
        (&mut Sprite, &ItemOf, &Equippable),
        (With<Shield>, With<ActiveShield>),
    >,
) {
    let Ok((mut shield_sprite, _, equippable)) = shield_query.get_mut(shield.item) else {
        warn!("Offhand missing Shield or ActiveShield");
        return;
    };

    let Ok(facing_direction) = holder_query.get(shield.holder) else {
        warn!("Tried to stop blocking but entity no facing direction");
        return;
    };

    commands
        .entity(shield.item)
        .remove::<(EquipmentInUse, ActiveShield, Collider, CollisionLayers)>()
        .insert(equippable.poses.transform(*facing_direction));

    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = 0;
    }
}
