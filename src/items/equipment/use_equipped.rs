use bevy::prelude::*;

use super::EquipmentSlot;
use crate::{
    combat::{Mana, mana::ManaCost},
    items::{
        Items,
        equipment::{Equippable, Equipped, Mainhand, Offhand},
    },
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
