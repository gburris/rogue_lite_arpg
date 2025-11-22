use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, FRAC_PI_4, PI},
};

use avian2d::prelude::*;
use bevy::{prelude::*, ui_widgets::observe};

use crate::{
    items::{
        Item, ItemOf, ItemType,
        equipment::{EquipmentSlot, EquipmentType, Equippable, Equipped},
        prelude::UseEquipment,
    },
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_active_shields, reflect_projectiles).in_set(InGameSystems::Simulation),
    );
}

pub fn magic_shield(
    sprites: &Res<SpriteAssets>,
    sprite_layouts: &Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Name::new("Magic Shield"),
        Item::new(355, ItemType::Tome),
        Equippable::from(0.5, EquipmentSlot::Offhand, EquipmentType::Shield),
        ManaCost(5.0),
        ManaDrainRate(20.0),
        ProjectileReflection,
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        Sprite {
            image: sprites.magic_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        observe(on_shield_block),
        observe(on_shield_deactivated),
    )
}

pub fn knight_shield(
    sprites: &Res<SpriteAssets>,
    sprite_layouts: &Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Name::new("Knight Shield"),
        Item::new(355, ItemType::Tome),
        Equippable::from(0.5, EquipmentSlot::Offhand, EquipmentType::Shield),
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        ManaDrainRate(25.0),
        ManaCost(25.0),
        Sprite {
            image: sprites.knight_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        observe(on_shield_block),
        observe(on_shield_deactivated),
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
pub struct ActiveShield {
    pub projectiles_reflected: HashSet<Entity>,
}

fn update_active_shields(
    mut commands: Commands,
    time: Res<Time>,
    mut active_shield_query: Query<
        (Entity, &ManaDrainRate, &ItemOf, &mut Sprite),
        (With<ActiveShield>, With<Equipped>),
    >,
    mut holder_query: Query<(&Vision, Option<&mut Mana>)>,
) -> Result {
    for (shield_entity, mana_drain_rate, item_of, mut shield_sprite) in &mut active_shield_query {
        let (vision, mana) = holder_query.get_mut(item_of.0)?;

        if let Some(mut mana) = mana {
            let drain_amount = ManaCost(mana_drain_rate.0 * time.delta_secs());

            if mana.has_enough_mana(&drain_amount) {
                mana.use_mana(&drain_amount);
            } else {
                commands.trigger(StopUsingEquipment {
                    entity: shield_entity,
                });
                continue;
            }
        }

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

fn on_shield_block(
    used_shield: On<UseEquipment>,
    mut commands: Commands,
    mut shield_query: Query<&Shield>,
) {
    let Ok(shield) = shield_query.get_mut(used_shield.entity) else {
        warn!("Tried to block with invalid shield");
        return;
    };

    commands.entity(used_shield.entity).insert((
        ActiveShield {
            projectiles_reflected: HashSet::default(),
        },
        shield.hitbox.clone(),
        ProjectileReflection::collision_layers(),
    ));
}

fn reflect_projectiles(
    mut shield_query: Query<
        (&mut ActiveShield, &CollidingEntities, &ChildOf),
        With<ProjectileReflection>,
    >,
    mut projectile_query: Query<
        (&mut LinearVelocity, &mut CollisionLayers, &mut Transform),
        With<Projectile>,
    >,
    enemy_query: Query<&Enemy>,
) {
    for (mut shield, colliding_entities, child_of) in &mut shield_query {
        for &colliding_entity in colliding_entities.iter() {
            if shield.projectiles_reflected.contains(&colliding_entity) {
                continue;
            }
            if let Ok((mut linear_velocity, mut collision_layers, mut transform)) =
                projectile_query.get_mut(colliding_entity)
            {
                // If holder is enemy and it is reflected, it can now hurt the player!
                let new_damage_source = if enemy_query.contains(child_of.parent()) {
                    DamageSource::Enemy
                } else {
                    DamageSource::Player
                };

                // Reverse direction of projectile! Reflect!
                linear_velocity.0 = -linear_velocity.0;

                // Rotate projectile sprite to face new velocity direction
                transform.rotation = Quat::from_rotation_z(linear_velocity.0.to_angle());

                *collision_layers = CollisionLayers::new(
                    GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                    LayerMask::from(new_damage_source) | GameCollisionLayer::HighObstacle,
                );
                shield.projectiles_reflected.insert(colliding_entity);
            }
        }
    }
}

fn on_shield_deactivated(
    shield: On<StopUsingEquipment>,
    mut commands: Commands,
    holder_query: Query<&FacingDirection>,
    mut shield_query: Query<(&mut Sprite, &ItemOf), (With<Shield>, With<ActiveShield>)>,
) {
    let Ok((mut shield_sprite, item_of)) = shield_query.get_mut(shield.entity) else {
        warn!("Offhand missing Shield or ActiveShield");
        return;
    };

    let Ok(facing_direction) = holder_query.get(item_of.0) else {
        warn!("Tried to stop blocking but entity no facing direction");
        return;
    };

    commands
        .entity(shield.entity)
        .remove::<(ActiveShield, Collider)>();
    // .insert(
    //     EquipmentTransform::get(*facing_direction)
    //         .expect("Failed to deactivate shield")
    //         .offhand,
    // );

    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = 0;
    }
}
