use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    prelude::*,
};

use crate::{
    character::Character,
    items::{
        ItemOf,
        equipment::{Equippable, Equipped},
    },
    prelude::{EquipmentSlot, Mainhand, Mana, ManaCost, Offhand},
};

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
/// Request to start using an equipped item. Item start observers own inserting `EquipmentInUse`.
#[derive(EntityEvent)]
pub struct StartUsingEquipment {
    #[event_target]
    pub item: Entity,
    pub holder: Entity,
    pub slot: EquipmentSlot,
}
/// Present on an equipped item while its use is in progress (swinging, casting, blocking).
/// Inserted by the item's `StartUsingEquipment` observer, removed by its stop path.
#[derive(Component, Default)]
pub struct EquipmentInUse;

/// Request to end an in-progress use. Item stop observers own removing `EquipmentInUse`.
#[derive(EntityEvent)]
pub struct StopUsingEquipment {
    #[event_target]
    pub item: Entity,
    pub holder: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentUseFailure {
    OutOfMana,
    OnCooldown,
    /// Holder entity is not a character, so it has no equipment slots.
    NoHolder,
    /// The requested slot has no equipped item.
    EmptySlot,
    /// The equipped item lacks usable equipment data (`Equippable`).
    UnusableItem,
}

#[derive(EntityEvent)]
pub struct EquipmentUseFailed {
    #[event_target]
    pub holder: Entity,
    pub slot: EquipmentSlot,
    pub reason: EquipmentUseFailure,
}
/// Fired at the holder alongside `StartUsingEquipment` so presentation scoped to the
/// holder (e.g. the player action bar) reacts without re-resolving the slot.
#[derive(EntityEvent)]
pub struct EquipmentUseSucceeded {
    #[event_target]
    pub holder: Entity,
    pub slot: EquipmentSlot,
    pub item: Entity,
}

/// Advances every equipped item's cooldown timer.
pub fn tick_equippable_cooldown(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in &mut equippable_query {
        equippable.cooldown.tick(time.delta());
    }
}

/// Query for an equipped item that can be used (cooldown, mana cost, owner).
#[derive(QueryData)]
#[query_data(mutable)]
pub struct UsableItem {
    // It is required that all reference lifetimes are explicitly annotated, just like in any
    // struct. Each lifetime should be 'static.
    pub equippable: &'static mut Equippable,
    equipped: &'static Equipped, // just a marker component
    mana_cost: Option<&'static ManaCost>,
    pub item_of: &'static ItemOf,
}

impl UsableItemItem<'_, '_> {
    /// Whether the item is ready and affordable, without consuming anything.
    pub fn can_use(&self, holder_mana: Option<&Mana>) -> Result<(), EquipmentUseFailure> {
        can_use_item(&self.equippable, self.mana_cost, holder_mana, true)
    }

    /// Checks every gate before consuming mana or resetting the cooldown.
    pub fn consume(&mut self, holder_mana: Option<&mut Mana>) -> Result<(), EquipmentUseFailure> {
        let can_use = self.can_use(holder_mana.as_deref());

        if can_use.is_ok() {
            if let (Some(holder_mana), Some(mana_cost)) = (holder_mana, self.mana_cost) {
                holder_mana.use_mana(mana_cost);
            }

            self.equippable.cooldown.reset();
        }

        can_use
    }
}

impl UsableItemReadOnlyItem<'_, '_> {
    /// Whether the item is ready and affordable, without consuming anything.
    pub fn can_use(&self, holder_mana: Option<&Mana>) -> Result<(), EquipmentUseFailure> {
        can_use_item(self.equippable, self.mana_cost, holder_mana, false)
    }
}

/// Shared readiness gate for both query views: cooldown first, then mana.
/// `report_missing_mana` is false for passive planning checks so an AI
/// inspecting a slot does not log a player-facing warning every tick.
fn can_use_item(
    equippable: &Equippable,
    mana_cost: Option<&ManaCost>,
    holder_mana: Option<&Mana>,
    report_missing_mana: bool,
) -> Result<(), EquipmentUseFailure> {
    if !equippable.cooldown.is_finished() {
        return Err(EquipmentUseFailure::OnCooldown);
    }
    check_mana(mana_cost, holder_mana, report_missing_mana)
}

fn check_mana(
    mana_cost: Option<&ManaCost>,
    mana: Option<&Mana>,
    report_missing_mana: bool,
) -> Result<(), EquipmentUseFailure> {
    if let (Some(mana), Some(mana_cost)) = (mana, mana_cost) {
        if !mana.has_enough_mana(mana_cost) {
            if report_missing_mana {
                debug!("Not enough mana!");
            }
            return Err(EquipmentUseFailure::OutOfMana);
        }
    } else if mana.is_none() && mana_cost.is_some() {
        if report_missing_mana {
            warn!("This wielder is not skilled in the arts of the arcane");
        }
        return Err(EquipmentUseFailure::OutOfMana);
    }
    Ok(())
}
/// Shared capability for using equipped items: resolves
/// character -> requested slot -> equipped item -> cooldown and mana.
#[derive(SystemParam)]
pub struct EquipmentUsage<'w, 's> {
    commands: Commands<'w, 's>,
    holders: Query<
        'w,
        's,
        (
            Option<&'static Mainhand>,
            Option<&'static Offhand>,
            Option<&'static mut Mana>,
        ),
        With<Character>,
    >,
    equipment: Query<'w, 's, UsableItem>,
    in_use: Query<'w, 's, (), With<EquipmentInUse>>,
}

impl EquipmentUsage<'_, '_> {
    /// Item currently equipped in the holder's slot, if any.
    pub fn item_in_slot(&self, holder: Entity, slot: EquipmentSlot) -> Option<Entity> {
        let (mainhand, offhand, _) = self.holders.get(holder).ok()?;
        slot.get_equipped(mainhand, offhand)
    }

    /// Whether the holder could use the slot now, without consuming anything.
    pub fn can_use(&self, holder: Entity, slot: EquipmentSlot) -> Result<(), EquipmentUseFailure> {
        let (mainhand, offhand, mana) = self
            .holders
            .get(holder)
            .map_err(|_| EquipmentUseFailure::NoHolder)?;
        let item = slot
            .get_equipped(mainhand, offhand)
            .ok_or(EquipmentUseFailure::EmptySlot)?;
        let usable = self
            .equipment
            .get(item)
            .map_err(|_| EquipmentUseFailure::UnusableItem)?;
        usable.can_use(mana)
    }

    /// Attempts to use the holder's slot, queuing `StartUsingEquipment` (targeted at
    /// the item) plus `EquipmentUseSucceeded` (targeted at the holder) or a
    /// failure event, and returns the real result to the caller.
    pub fn try_use(
        &mut self,
        holder: Entity,
        slot: EquipmentSlot,
    ) -> Result<Entity, EquipmentUseFailure> {
        let result = self.consume_and_resolve(holder, slot);
        match result {
            Ok(item) => {
                self.commands
                    .trigger(StartUsingEquipment { item, holder, slot });
                self.commands
                    .trigger(EquipmentUseSucceeded { holder, slot, item });
            }
            Err(reason) => {
                self.commands.trigger(EquipmentUseFailed {
                    holder,
                    slot,
                    reason,
                });
            }
        }
        result
    }

    // Split out so every early return maps to a failure and nothing is queued on failure.
    fn consume_and_resolve(
        &mut self,
        holder: Entity,
        slot: EquipmentSlot,
    ) -> Result<Entity, EquipmentUseFailure> {
        let (mainhand, offhand, mana) = self
            .holders
            .get_mut(holder)
            .map_err(|_| EquipmentUseFailure::NoHolder)?;
        let item = slot
            .get_equipped(mainhand, offhand)
            .ok_or(EquipmentUseFailure::EmptySlot)?;
        let mut usable = self
            .equipment
            .get_mut(item)
            .map_err(|_| EquipmentUseFailure::UnusableItem)?;
        usable.consume(mana.map(Mut::into_inner))?;
        Ok(item)
    }

    /// Queues `StopUsingEquipment` if the slot's item is in use. Returns whether it did.
    pub fn stop_using(&mut self, holder: Entity, slot: EquipmentSlot) -> bool {
        match self.item_in_slot(holder, slot) {
            Some(item) if self.in_use.contains(item) => {
                self.commands.trigger(StopUsingEquipment { item, holder });
                true
            }
            _ => false,
        }
    }

    /// Whether the item in the slot currently carries `EquipmentInUse` (swinging, casting, blocking).
    pub fn is_using(&self, holder: Entity, slot: EquipmentSlot) -> bool {
        self.item_in_slot(holder, slot)
            .is_some_and(|item| self.in_use.contains(item))
    }

    /// The usable item data equipped in the holder's slot, if any.
    pub fn equippable_in_slot(&self, holder: Entity, slot: EquipmentSlot) -> Option<&Equippable> {
        let item = self.item_in_slot(holder, slot)?;
        self.equipment
            .get(item)
            .ok()
            .map(|usable| usable.equippable)
    }
}

/// Read-only view of equipment state for planners. It never requests mutable
/// `Mana` or `Equippable`, so decision systems can run alongside regeneration
/// and cooldown ticks without conflicts.
#[derive(SystemParam)]
pub struct EquipmentRead<'w, 's> {
    holders: Query<
        'w,
        's,
        (
            Option<&'static Mainhand>,
            Option<&'static Offhand>,
            Option<&'static Mana>,
        ),
        With<Character>,
    >,
    equipment: Query<'w, 's, UsableItemReadOnly>,
}

impl EquipmentRead<'_, '_> {
    /// Item currently equipped in the holder's slot, if any.
    pub fn item_in_slot(&self, holder: Entity, slot: EquipmentSlot) -> Option<Entity> {
        let (mainhand, offhand, _) = self.holders.get(holder).ok()?;
        slot.get_equipped(mainhand, offhand)
    }

    /// The usable item data equipped in the holder's slot, if any.
    pub fn equippable_in_slot(&self, holder: Entity, slot: EquipmentSlot) -> Option<&Equippable> {
        let item = self.item_in_slot(holder, slot)?;
        self.equipment
            .get(item)
            .ok()
            .map(|usable| usable.equippable)
    }

    /// Whether the holder could use the slot now, without consuming anything.
    pub fn can_use(&self, holder: Entity, slot: EquipmentSlot) -> Result<(), EquipmentUseFailure> {
        let (mainhand, offhand, mana) = self
            .holders
            .get(holder)
            .map_err(|_| EquipmentUseFailure::NoHolder)?;
        let item = slot
            .get_equipped(mainhand, offhand)
            .ok_or(EquipmentUseFailure::EmptySlot)?;
        let usable = self
            .equipment
            .get(item)
            .map_err(|_| EquipmentUseFailure::UnusableItem)?;
        usable.can_use(mana)
    }
}
