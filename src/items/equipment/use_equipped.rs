use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use rand::Rng;

use super::EquipmentSlot;
use crate::{
    animation::heal_vfx,
    combat::{
        Mana, Projectile,
        damage::DamageSource,
        health::AttemptHeal,
        mana::ManaCost,
        melee::{MeleeWeapon, start_melee_attack},
        projectile::{ProjectileOf, Projectiles},
        shield::{ActiveShield, shield_block::deactivate_shield},
        status_effects::Effects,
    },
    configuration::{
        GameCollisionLayer, ZLayer,
        assets::{SpriteAssets, SpriteSheetLayouts},
    },
    items::{
        HealingTome, Items, Shield,
        equipment::{Equippable, Equipped, Mainhand, Offhand},
    },
    prelude::{Enemy, *},
};

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
#[derive(EntityEvent)]
pub struct UseEquipment {
    pub entity: Entity,
    pub holder: Entity,
}

#[derive(EntityEvent)]
pub struct UseEquipmentInput {
    pub entity: Entity,
    pub slot: EquipmentSlot,
}

#[derive(EntityEvent)]
pub struct StopUsingHoldableEquipmentInput {
    pub entity: Entity,
    pub slot: EquipmentSlot,
}

#[derive(PartialEq)]
pub enum EquipmentUseFailure {
    OutOfMana,
    OnCooldown,
    NoneEquipped,
}

#[derive(EntityEvent)]

pub struct EquipmentUseFailed {
    pub entity: Entity,
    pub slot: EquipmentSlot,
    pub reason: EquipmentUseFailure,
}

pub fn tick_equippable_use_rate(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in equippable_query.iter_mut() {
        equippable.use_rate.tick(time.delta());
    }
}
pub fn on_equipment_activated(
    equipment_used: On<UseEquipmentInput>,
    commands: Commands,
    holder_query: Query<(Option<&mut Mana>, Option<&Mainhand>, Option<&Offhand>), With<Items>>,
    equippable_query: Query<(&mut Equippable, Option<&ManaCost>), With<Equipped>>,
) {
    handle_equipment_activation(
        equipment_used.entity,
        equipment_used.slot,
        commands,
        holder_query,
        equippable_query,
    );
}

fn handle_equipment_activation(
    entity: Entity,
    slot: EquipmentSlot,
    mut commands: Commands,
    mut holder_query: Query<(Option<&mut Mana>, Option<&Mainhand>, Option<&Offhand>), With<Items>>,
    mut equippable_query: Query<(&mut Equippable, Option<&ManaCost>), With<Equipped>>,
) {
    let Ok((mut holder_mana, mainhand, offhand)) = holder_query.get_mut(entity) else {
        error!("Entity: {} tried to use equipment, but has none", entity);
        return;
    };

    let equipment_entity: Option<Entity> = match slot {
        EquipmentSlot::Mainhand => mainhand.map(|m| m.0),
        EquipmentSlot::Offhand => offhand.map(|o| o.0),
    };

    let Some(equipment_entity) = equipment_entity else {
        warn!("{:?} is empty!", slot);
        commands.trigger(EquipmentUseFailed {
            entity,
            slot,
            reason: EquipmentUseFailure::NoneEquipped,
        });
        return;
    };

    if let Ok((mut equippable, mana_cost)) = equippable_query.get_mut(equipment_entity) {
        // Check cooldown first
        if !equippable.use_rate.is_finished() {
            commands.trigger(EquipmentUseFailed {
                entity,
                slot,
                reason: EquipmentUseFailure::OnCooldown,
            });
            return;
        }

        // Check mana next
        if let (Some(mana), Some(mana_cost)) = (holder_mana.as_mut(), mana_cost) {
            if !mana.attempt_use_mana(mana_cost) {
                debug!("Not enough mana!");
                commands.trigger(EquipmentUseFailed {
                    entity,
                    slot,
                    reason: EquipmentUseFailure::OutOfMana,
                });
                return;
            }
        } else if holder_mana.is_none() && mana_cost.is_some() {
            warn!("This wielder is not skilled in the arts of the arcane");
            return;
        }

        // Success path - trigger equipment use and reset cooldown
        commands.trigger(UseEquipment {
            entity: equipment_entity,
            holder: entity,
        });
        equippable.use_rate.reset();
    }
}

// "fired" implies this is a projectile weapon
pub fn on_weapon_fired(
    weapon_fired: On<UseEquipment>,
    mut commands: Commands,
    weapon_query: Query<&Projectiles>,
    holder_query: Query<(&Transform, &Vision)>,
    enemy_query: Query<Entity, With<Enemy>>,
    projectile_query: Query<(&Projectile, Option<&Effects>), With<Disabled>>,
) {
    let Ok(projectiles) = weapon_query.get(weapon_fired.entity) else {
        warn!("Tried to fire weapon that is not a projectile weapon");
        return;
    };

    let damage_source = if enemy_query.get(weapon_fired.holder).is_ok() {
        DamageSource::Enemy
    } else {
        DamageSource::Player
    };

    let Ok((holder_transform, holder_vision)) = holder_query.get(weapon_fired.holder) else {
        warn!("Tried to fire weapon with holder missing aim position or transform");
        return;
    };

    for projectile_entity in projectiles.iter() {
        if let Ok((projectile, effects)) = projectile_query.get(projectile_entity) {
            trace!("Spawning projectile with effects: {:?}", effects);

            // Rotate the aim direction by the projectile’s angle offset
            let rotated_direction = holder_vision
                .aim_direction
                .rotate(Vec2::from_angle(projectile.angle_offset));
            let starting_position = holder_transform.translation.truncate()
                + (projectile.forward_offset * rotated_direction);

            commands
                .entity(projectile_entity)
                .clone_and_spawn_with_opt_out(|builder| {
                    //builder.deny::<(Position, Rotation)>();
                    builder.linked_cloning(true);
                })
                .remove::<(ProjectileOf, Disabled)>()
                .insert((
                    Position(starting_position),
                    Rotation::radians(rotated_direction.to_angle()),
                    Transform {
                        translation: starting_position.extend(ZLayer::InAir.z()),
                        rotation: Quat::from_rotation_z(rotated_direction.to_angle()),
                        ..default()
                    },
                    LinearVelocity(rotated_direction * projectile.speed),
                    CollisionLayers::new(
                        GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                        LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
                    ),
                ));
        }
    }
}

pub fn on_weapon_melee(
    melee_weapon_used: On<UseEquipment>,
    mut commands: Commands,
    mut weapon_query: Query<(Entity, &mut MeleeWeapon)>,
    mut action_state_query: Query<&mut ActionState>,
    holder_query: Query<&Vision>,
) {
    let Ok((weapon_entity, mut melee_weapon)) = weapon_query.get_mut(melee_weapon_used.entity)
    else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok(vision) = holder_query.get(melee_weapon_used.holder) else {
        warn!("Holder missing required components");
        return;
    };

    let attack_angle = vision.aim_direction.to_angle();

    start_melee_attack(
        &mut commands,
        weapon_entity,
        &mut melee_weapon,
        attack_angle,
    );

    if let Ok(mut action_state) = action_state_query.get_mut(melee_weapon_used.holder) {
        *action_state = ActionState::Attacking;
    }
}

pub fn on_healing_tome_cast(
    use_healing_tome: On<UseEquipment>,
    mut commands: Commands,
    tome_query: Query<&HealingTome>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    let tome_entity = use_healing_tome.entity;
    let holder_entity = use_healing_tome.holder;

    let Ok(tome) = tome_query.get(tome_entity) else {
        warn!("Tried to use a tome that does not exist");
        return;
    };

    let health_to_add = rand::rng().random_range(tome.healing.0..tome.healing.1);
    commands.trigger(AttemptHeal {
        entity: holder_entity,
        amount: health_to_add,
    });
    commands
        .entity(holder_entity)
        .with_child(heal_vfx(sprites, sprite_layouts));
}

pub fn on_shield_block(
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

pub fn on_equipment_deactivated(
    stop_using_holdable: On<StopUsingHoldableEquipmentInput>,
    mut commands: Commands,
    holder_query: Query<(&Offhand, &FacingDirection)>,
    mut shield_query: Query<&mut Sprite, (With<Shield>, With<ActiveShield>)>,
) {
    // Get the holder's inventory
    let Ok((Offhand(shield_entity), facing_direction)) =
        holder_query.get(stop_using_holdable.entity)
    else {
        warn!("Tried to stop blocking but entity has no offhand or no direction");
        return;
    };

    if let Ok(mut shield_sprite) = shield_query.get_mut(*shield_entity) {
        deactivate_shield(
            &mut commands,
            *shield_entity,
            *facing_direction,
            &mut shield_sprite,
        );
    } else {
        warn!("Offhand missing Shield or ActiveShield");
    }
}
