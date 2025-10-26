use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, FRAC_PI_4, PI},
};

use avian2d::prelude::*;
use bevy::{prelude::*, ui_widgets::observe};

use crate::{
    combat::{
        Mana, Projectile,
        damage::DamageSource,
        mana::{ManaCost, ManaDrainRate},
    },
    items::{
        Item, ItemOf, ItemType,
        equipment::{EquipmentSlot, Equippable, Equipped, Offhand},
        prelude::{EquipmentTransform, StopUsingHoldableEquipmentInput, UseEquipment},
    },
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_active_shields, reflect_projectiles).in_set(InGameSystems::Simulation),
    );

    app.add_observer(activate_shield)
        .add_observer(on_shield_deactivated);
}

pub fn magic_shield(
    sprites: &Res<SpriteAssets>,
    sprite_layouts: &Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Name::new("Magic Shield"),
        Item::new(355, ItemType::Tome),
        Equippable::from(0.5, EquipmentSlot::Offhand),
        ManaCost(5.0),
        ManaDrainRate(20.0),
        ProjectileReflection,
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        Holdable,
        Sprite {
            image: sprites.magic_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        observe(on_shield_block),
    )
}

pub fn knight_shield(
    sprites: &Res<SpriteAssets>,
    sprite_layouts: &Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Name::new("Knight Shield"),
        Item::new(355, ItemType::Tome),
        Equippable::from(0.5, EquipmentSlot::Offhand),
        Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        },
        ManaDrainRate(25.0),
        ManaCost(25.0),
        Holdable,
        Sprite {
            image: sprites.knight_shield.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.shield_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        observe(on_shield_block),
    )
}

#[derive(Component)]
#[require(Holdable)]
pub struct Shield {
    pub hitbox: Collider,
}

//This component tags items that are active continiously while being used
//e.g. Holding right will keep a shield up
#[derive(Component, Default)]
pub struct Holdable;

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

fn activate_shield(
    active_shield: On<Add, ActiveShield>,
    mut commands: Commands,
    shield_query: Query<&Shield>,
) {
    if let Ok(activated_shield) = shield_query.get(active_shield.entity) {
        commands.entity(active_shield.entity).insert((
            activated_shield.hitbox.clone(),
            ProjectileReflection::collision_layers(),
        ));
    } else {
        warn!("Active Shield added to something that isn't a shield");
    }
}

fn deactivate_shield(
    commands: &mut Commands,
    shield_entity: Entity,
    facing_direction: FacingDirection,
    shield_sprite: &mut Sprite,
) -> Result {
    commands
        .entity(shield_entity)
        .remove::<(ActiveShield, Collider)>()
        .insert(EquipmentTransform::get(facing_direction)?.offhand);

    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = 0;
    }
    Ok(())
}

fn update_active_shields(
    mut commands: Commands,
    time: Res<Time>,
    mut active_shield_query: Query<
        (Entity, &ManaDrainRate, &ItemOf, &mut Sprite),
        (With<ActiveShield>, With<Equipped>),
    >,
    mut holder_query: Query<(&Vision, &FacingDirection, Option<&mut Mana>)>,
) -> Result {
    for (shield_entity, mana_drain_rate, item_of, mut shield_sprite) in
        active_shield_query.iter_mut()
    {
        let (vision, facing_direction, mana) = holder_query.get_mut(item_of.0)?;

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

        if let Some(mut mana) = mana {
            let drain_amount = ManaCost(mana_drain_rate.0 * time.delta_secs());
            if !mana.attempt_use_mana(&drain_amount) {
                deactivate_shield(
                    &mut commands,
                    shield_entity,
                    *facing_direction,
                    &mut shield_sprite,
                )?;
            }
        }
    }
    Ok(())
}

fn on_shield_block(
    use_shield_block: On<UseEquipment>,
    mut commands: Commands,
    mut shield_query: Query<(Entity, &Shield)>,
) {
    let Ok((shield_entity, _)) = shield_query.get_mut(use_shield_block.entity) else {
        warn!("Tried to block with invalid shield");
        return;
    };
    commands.entity(shield_entity).insert(ActiveShield {
        projectiles_reflected: Default::default(),
    });
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
    for (mut shield, colliding_entities, child_of) in shield_query.iter_mut() {
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
    stop_using_holdable: On<StopUsingHoldableEquipmentInput>,
    mut commands: Commands,
    holder_query: Query<(&Offhand, &FacingDirection)>,
    mut shield_query: Query<&mut Sprite, (With<Shield>, With<ActiveShield>)>,
) {
    // Get the holder's inventory
    let Ok((offhand, facing_direction)) = holder_query.get(stop_using_holdable.entity) else {
        warn!("Tried to stop blocking but entity has no offhand or no direction");
        return;
    };

    if let Ok(mut shield_sprite) = shield_query.get_mut(offhand.get()) {
        let shield_result = deactivate_shield(
            &mut commands,
            offhand.get(),
            *facing_direction,
            &mut shield_sprite,
        );
        if let Err(e) = shield_result {
            warn!("Failed to deactivate shield: {}", e);
        }
    } else {
        warn!("Offhand missing Shield or ActiveShield");
    }
}
