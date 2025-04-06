use bevy::prelude::*;
use rand::Rng;

use super::{EquipmentSlot, Equipped};
use crate::{
    ai::state::{ActionState, AimPosition, FacingDirection},
    combat::{
        damage::DamageSource,
        health::AttemptHealingEvent,
        mana::ManaCost,
        melee::{start_melee_attack, MeleeWeapon},
        projectile::{spawn::spawn_projectile, ProjectileWeapon},
        shield::{shield_block::deactivate_shield, ActiveShield},
        Mana,
    },
    enemy::Enemy,
    items::{
        equipment::Equippable, inventory::Inventory, HealingTome, HealingTomeSpellVisualEffect,
        Shield,
    },
    player::{StopUsingHoldableEquipmentInputEvent, UseEquipmentInputEvent},
};

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
#[derive(Event)]
pub struct UseEquipmentEvent {
    pub holder: Entity,
}

#[derive(PartialEq)]
pub enum EquipmentUseFailure {
    OutOfMana,
    OnCooldown,
    NoneEquipped,
}

#[derive(Event)]

pub struct EquipmentUseFailedEvent {
    pub holder: Entity,
    pub slot: EquipmentSlot,
    pub reason: EquipmentUseFailure,
}

pub fn tick_equippable_use_rate(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in equippable_query.iter_mut() {
        equippable.use_rate.tick(time.delta());
    }
}
pub fn on_equipment_activated(
    trigger: Trigger<UseEquipmentInputEvent>,
    commands: Commands,
    holder_query: Query<(&Inventory, Option<&mut Mana>)>,
    equippable_query: Query<(&mut Equippable, Option<&ManaCost>), With<Equipped>>,
) {
    handle_equipment_activation(
        trigger.target(),
        trigger.slot,
        commands,
        holder_query,
        equippable_query,
    );
}

fn handle_equipment_activation(
    entity: Entity,
    slot: EquipmentSlot,
    mut commands: Commands,
    mut holder_query: Query<(&Inventory, Option<&mut Mana>)>,
    mut equippable_query: Query<(&mut Equippable, Option<&ManaCost>), With<Equipped>>,
) {
    let Ok((inventory, mut holder_mana)) = holder_query.get_mut(entity) else {
        error!(
            "Entity: {} tried to use equipment, but is missing inventory",
            entity
        );
        return;
    };

    let Some(equipment_entity) = inventory.get_equipped(slot) else {
        warn!("{:?} is empty!", slot);
        commands.trigger_targets(
            EquipmentUseFailedEvent {
                holder: entity,
                slot,
                reason: EquipmentUseFailure::NoneEquipped,
            },
            entity,
        );
        return;
    };

    if let Ok((mut equippable, mana_cost)) = equippable_query.get_mut(equipment_entity) {
        // Check cooldown first
        if !equippable.use_rate.finished() {
            commands.trigger_targets(
                EquipmentUseFailedEvent {
                    holder: entity,
                    slot,
                    reason: EquipmentUseFailure::OnCooldown,
                },
                entity,
            );
            return;
        }

        // Check mana next
        if let (Some(mana), Some(mana_cost)) = (holder_mana.as_mut(), mana_cost) {
            if !mana.attempt_use_mana(mana_cost) {
                warn!("Not enough mana!");
                commands.trigger_targets(
                    EquipmentUseFailedEvent {
                        holder: entity,
                        slot,
                        reason: EquipmentUseFailure::OutOfMana,
                    },
                    entity,
                );
                return;
            }
        } else if holder_mana.is_none() && mana_cost.is_some() {
            warn!("This wielder is not skilled in the arts of the arcane");
            return;
        }

        // Success path - trigger equipment use and reset cooldown
        commands.trigger_targets(UseEquipmentEvent { holder: entity }, equipment_entity);
        equippable.use_rate.reset();
    }
}

// "fired" implies this is a projectile weapon
pub fn on_weapon_fired(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    weapon_query: Query<&ProjectileWeapon>,
    holder_query: Query<(&Transform, &AimPosition)>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    let mut damage_source = DamageSource::Player;
    let Ok(projectile_weapon) = weapon_query.get(fired_trigger.target()) else {
        warn!("Tried to fire weapon that is not a projectile weapon");
        return;
    };
    if let Ok(_enemy) = enemy_query.get(fired_trigger.holder) {
        damage_source = DamageSource::Enemy;
    }
    let Ok((holder_transform, holder_aim)) = holder_query.get(fired_trigger.holder) else {
        warn!("Tried to fire weapon with holder missing aim position or transform");
        return;
    };

    spawn_projectile(
        damage_source,
        &mut commands,
        holder_transform,
        holder_aim.position,
        projectile_weapon,
    );
}

pub fn on_weapon_melee(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    mut weapon_query: Query<(Entity, &mut MeleeWeapon)>,
    mut action_state_query: Query<&mut ActionState>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    let Ok((weapon_entity, mut melee_weapon)) = weapon_query.get_mut(fired_trigger.target()) else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok((holder_transform, aim_pos)) = holder_query.get(fired_trigger.holder) else {
        warn!("Holder missing required components");
        return;
    };

    let holder_pos = holder_transform.translation.truncate();
    let aim_direction: Vec2 = (aim_pos.position - holder_pos).normalize();
    let attack_angle = aim_direction.y.atan2(aim_direction.x);

    start_melee_attack(
        &mut commands,
        weapon_entity,
        &mut melee_weapon,
        attack_angle,
    );

    if let Ok(mut action_state) = action_state_query.get_mut(fired_trigger.holder) {
        *action_state = ActionState::Attacking;
    }
}

pub fn on_healing_tome_cast(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    tome_query: Query<&HealingTome>,
) {
    let Ok(tome) = tome_query.get(fired_trigger.target()) else {
        warn!("Tried to use a tome that does not exist");
        return;
    };

    let health_to_add = rand::thread_rng().gen_range(tome.healing.0..tome.healing.1);
    commands.trigger_targets(
        AttemptHealingEvent {
            amount: health_to_add,
        },
        fired_trigger.holder,
    );
    commands
        .entity(fired_trigger.holder)
        .with_child(HealingTomeSpellVisualEffect);
}

pub fn on_shield_block(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    mut shield_query: Query<(Entity, &Shield)>,
) {
    let Ok((shield_entity, _)) = shield_query.get_mut(fired_trigger.target()) else {
        warn!("Tried to block with invalid shield");
        return;
    };
    commands.entity(shield_entity).insert(ActiveShield {
        projectiles_reflected: Default::default(),
    });
}

pub fn on_equipment_deactivated(
    fired_trigger: Trigger<StopUsingHoldableEquipmentInputEvent>,
    mut commands: Commands,
    holder_query: Query<(&Inventory, &FacingDirection)>,
    mut shield_query: Query<(Entity, &mut Sprite), (With<Shield>, With<ActiveShield>)>,
) {
    // Get the holder's inventory
    let Ok((inventory, facing_direction)) = holder_query.get(fired_trigger.target()) else {
        warn!("Tried to stop blocking but entity has no inventory or no direction");
        return;
    };

    let Some(shield_entity) = inventory.get_equipped(EquipmentSlot::Offhand) else {
        warn!("No shield equipped in offhand");
        return;
    };
    if let Ok((shield_entity, mut shield_sprite)) = shield_query.get_mut(shield_entity) {
        deactivate_shield(
            &mut commands,
            shield_entity,
            *facing_direction,
            Some(&mut shield_sprite),
        );
    } else {
        warn!("Shield is equipped but doesn't have ActiveShield");
    }
}
