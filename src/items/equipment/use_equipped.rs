use bevy::prelude::*;

use crate::{
    combat::{
        attributes::{mana::ManaCost, Mana},
        components::{ActionState, AimPosition},
        damage::components::DamageSource,
        melee::{components::MeleeWeapon, swing_melee_attacks::start_melee_attack},
        projectile::spawn::spawn_projectile,
        weapon::weapon::ProjectileWeapon,
    },
    enemy::Enemy,
    items::{equipment::Equippable, inventory::Inventory},
    player::{UseMainhandInputEvent, UseOffhandInputEvent},
};

use super::{EquipmentSlot, Equipped};

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
#[derive(Event)]
pub struct UseEquipmentEvent {
    pub holder: Entity, // entity holding the equipment
}

pub fn tick_equippable_use_rate(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in equippable_query.iter_mut() {
        equippable.use_rate.tick(time.delta());
    }
}

// TODO: All of the "warns" in this function should be shown to the player through UI so they know why using main hand failed
// TODO #2: I'm not convinced on main hand activated is the best function to validate a user is OOM or
// Their weapon is on cooldown
pub fn on_main_hand_activated(
    main_hand_trigger: Trigger<UseMainhandInputEvent>,
    mut commands: Commands,
    mut holder_query: Query<(&Inventory, Option<&mut Mana>)>,
    mut main_hand_query: Query<(&mut Equippable, Option<&ManaCost>), With<Equipped>>,
) {
    let Ok((inventory, mut holder_mana)) = holder_query.get_mut(main_hand_trigger.entity()) else {
        error!(
            "Entity: {} tried to use main hand, but is missing equipment slots",
            main_hand_trigger.entity()
        );
        return;
    };

    let Some(main_hand_entity) = inventory.get_equipped(EquipmentSlot::Mainhand) else {
        warn!("Main hand is empty!");
        return;
    };

    if let Ok((mut equippable, mana_cost)) = main_hand_query.get_mut(main_hand_entity) {
        if equippable.use_rate.finished() {
            // If the equipment uses mana, and we don't have enough, return
            if let (Some(mana), Some(mana_cost)) = (holder_mana.as_mut(), mana_cost) {
                if !mana.attempt_use_mana(mana_cost) {
                    warn!("Not enough mana!");
                    return;
                }
            } else if holder_mana.is_none() && mana_cost.is_some() {
                warn!("This wielder is not skilled in the arts of the arcane");
                return;
            }

            commands.trigger_targets(
                UseEquipmentEvent {
                    holder: main_hand_trigger.entity(),
                },
                main_hand_entity,
            );
            equippable.use_rate.reset();
        }
    }
}

pub fn on_off_hand_activated(
    off_hand_trigger: Trigger<UseOffhandInputEvent>,
    mut commands: Commands,
    mut holder_query: Query<(&Inventory, Option<&mut Mana>)>,
    mut main_hand_query: Query<(&mut Equippable, Option<&ManaCost>), With<Equipped>>,
) {
    let Ok((inventory, mut holder_mana)) = holder_query.get_mut(off_hand_trigger.entity()) else {
        error!(
            "Entity: {} tried to use off hand, but is missing equipment slots",
            off_hand_trigger.entity()
        );
        return;
    };

    let Some(off_hand_entity) = inventory.get_equipped(EquipmentSlot::Offhand) else {
        warn!("Off hand is empty!");
        return;
    };

    if let Ok((mut equippable, mana_cost)) = main_hand_query.get_mut(off_hand_entity) {
        if equippable.use_rate.finished() {
            // If the equipment uses mana, and we don't have enough, return
            if let (Some(mana), Some(mana_cost)) = (holder_mana.as_mut(), mana_cost) {
                if !mana.attempt_use_mana(mana_cost) {
                    warn!("Not enough mana!");
                    return;
                }
            } else if holder_mana.is_none() && mana_cost.is_some() {
                warn!("This wielder is not skilled in the arts of the arcane");
                return;
            }

            commands.trigger_targets(
                UseEquipmentEvent {
                    holder: off_hand_trigger.entity(),
                },
                off_hand_entity,
            );
            equippable.use_rate.reset();
        }
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
    let Ok(projectile_weapon) = weapon_query.get(fired_trigger.entity()) else {
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
        &projectile_weapon,
    );
}

pub fn on_weapon_melee(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    mut weapon_query: Query<(Entity, &mut MeleeWeapon)>,
    mut action_state_query: Query<&mut ActionState>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    let Ok((weapon_entity, mut melee_weapon)) = weapon_query.get_mut(fired_trigger.entity()) else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok((holder_transform, aim_pos)) = holder_query.get(fired_trigger.holder) else {
        warn!("Holder missing required components");
        return;
    };

    let holder_pos = holder_transform.translation.truncate();
    let aim_direction: Vec2 = (aim_pos.position - holder_pos).normalize();
    let mut attack_angle = aim_direction.y.atan2(aim_direction.x);
    attack_angle -= std::f32::consts::FRAC_PI_2;

    start_melee_attack(
        &mut commands,
        weapon_entity,
        &mut melee_weapon,
        attack_angle,
    );

    //TODO: Refactor action state stuff
    if let Ok(mut action_state) = action_state_query.get_mut(fired_trigger.holder) {
        *action_state = ActionState::Attacking;
    }
}
