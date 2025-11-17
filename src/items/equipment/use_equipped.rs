use bevy::{ecs::query::QueryData, prelude::*};

use crate::{
    character::Character,
    items::{
        ItemOf,
        equipment::{Equippable, Equipped},
    },
    prelude::{EquipmentSlot, Mana, ManaCost},
};

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
#[derive(EntityEvent)]
pub struct UseEquipment {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct AIUseEquipment {
    pub entity: Entity,
}

pub(super) fn on_ai_equipment_used(
    equipment: On<AIUseEquipment>,
    mut commands: Commands,
    mut equipment_query: Query<EquipmentUsed>,
    mut holder_query: Query<Option<&mut Mana>, With<Character>>,
) {
    let Ok(mut equipment_used) = equipment_query.get_mut(equipment.entity) else {
        debug!("AI killed while attempting to use equipment");
        return;
    };

    let Ok(mut mana) = holder_query.get_mut(equipment_used.item_of.0) else {
        warn!("Non-character attempted to use equipment");
        return;
    };

    let use_result = equipment_used.attempt_use(mana.as_deref_mut());

    if use_result.is_ok() {
        commands.trigger(UseEquipment {
            entity: equipment.entity,
        });
    }
}

#[derive(EntityEvent)]
pub struct StopUsingEquipment {
    pub entity: Entity,
}

#[derive(PartialEq)]
pub enum EquipmentUseFailure {
    OutOfMana,
    OnCooldown,
    NoneEquipped,
}

#[derive(EntityEvent)]

pub struct EquipmentUseFailed {
    #[event_target]
    pub holder: Entity,
    pub slot: EquipmentSlot,
    pub reason: EquipmentUseFailure,
}

pub(super) fn tick_equippable_use_rate(
    mut equippable_query: Query<&mut Equippable>,
    time: Res<Time>,
) {
    for mut equippable in &mut equippable_query {
        equippable.use_rate.tick(time.delta());
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct EquipmentUsed {
    // It is required that all reference lifetimes are explicitly annotated, just like in any
    // struct. Each lifetime should be 'static.
    pub equippable: &'static mut Equippable,
    equipped: &'static Equipped, // just a marker component
    mana_cost: Option<&'static ManaCost>,
    pub item_of: &'static ItemOf,
}

impl EquipmentUsedItem<'_, '_> {
    pub fn can_use(&self, holder_mana: Option<&Mana>) -> Result<(), EquipmentUseFailure> {
        // Check cooldown first
        if !self.equippable.use_rate.is_finished() {
            return Err(EquipmentUseFailure::OnCooldown);
        }

        // Check mana next
        if let (Some(mana), Some(mana_cost)) = (holder_mana, self.mana_cost) {
            if !mana.has_enough_mana(mana_cost) {
                debug!("Not enough mana!");
                return Err(EquipmentUseFailure::OutOfMana);
            }
        } else if holder_mana.is_none() && self.mana_cost.is_some() {
            warn!("This wielder is not skilled in the arts of the arcane");
            return Err(EquipmentUseFailure::OutOfMana);
        }

        Ok(())
    }

    pub fn attempt_use(
        &mut self,
        holder_mana: Option<&mut Mana>,
    ) -> Result<(), EquipmentUseFailure> {
        let can_use = self.can_use(holder_mana.as_deref());

        if can_use.is_ok() {
            if let (Some(holder_mana), Some(mana_cost)) = (holder_mana, self.mana_cost) {
                holder_mana.use_mana(mana_cost);
            }

            self.equippable.use_rate.reset();
        }

        can_use
    }
}
