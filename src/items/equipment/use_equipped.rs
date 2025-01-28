use bevy::prelude::*;

use crate::{
    combat::{
        attributes::{mana::ManaCost, Mana},
        melee::{components::MeleeWeapon, spawn::spawn_melee_attack},
        projectile::spawn::spawn_projectile,
        weapon::weapon::ProjectileWeapon,
    },
    configuration::assets::SpriteAssets,
    items::equipment::Equippable,
    player::{systems::AimPosition, MainHandActivated},
};

use super::equipment_slots::EquipmentSlots;

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
#[derive(Event)]
pub struct UseEquipmentEvent {
    pub holder: Entity, // entity holding the equipment
}

// TODO: All of the "warns" in this function should be shown to the player through UI so they know why using main hand failed
// TODO #2: I'm not convinced on main hand activated is the best function to validate a user is OOM or
// Their weapon is on cooldown
pub fn on_main_hand_activated(
    main_hand_trigger: Trigger<MainHandActivated>,
    mut commands: Commands,
    mut holder_query: Query<(&EquipmentSlots, Option<&mut Mana>)>,
    mut main_hand_query: Query<(&mut Equippable, Option<&ManaCost>)>,
) {
    let Ok((equipment_slots, mut holder_mana)) = holder_query.get_mut(main_hand_trigger.entity())
    else {
        error!(
            "Entity: {} tried to use main hand, but is missing equipment slots",
            main_hand_trigger.entity()
        );
        return;
    };

    let Some(main_hand_entity) = equipment_slots.mainhand else {
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

// "fired" implies this is a projectile weapon
pub fn on_weapon_fired(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    weapon_query: Query<&ProjectileWeapon>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    let Ok(projectile_weapon) = weapon_query.get(fired_trigger.entity()) else {
        warn!("Tried to fire weapon that is not a projectile weapon");
        return;
    };

    let Ok((holder_transform, holder_aim)) = holder_query.get(fired_trigger.holder) else {
        warn!("Tried to fire weapon with holder missing aim position or transform");
        return;
    };

    spawn_projectile(
        &mut commands,
        holder_transform,
        holder_aim.position,
        &projectile_weapon,
    );
}

pub fn on_weapon_melee(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    weapon_query: Query<&MeleeWeapon>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    let Ok(melee_weapon) = weapon_query.get(fired_trigger.entity()) else {
        warn!("Tried to melee swing a weapon that is not a melee weapon");
        return;
    };

    let Ok((holder_transform, holder_aim)) = holder_query.get(fired_trigger.holder) else {
        warn!("Tried to fire weapon with holder missing aim position or transform");
        return;
    };

    spawn_melee_attack(
        &mut commands,
        fired_trigger.entity(),
        holder_transform,
        holder_aim.position,
        &melee_weapon.melee_attack,
    );
}
