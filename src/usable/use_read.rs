use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Usable;
use super::in_use::{InUse, StopUse, UseCooldown, UseStyle};
use crate::prelude::*;

/// Where a character finds a [`Usable`]: the item equipped in a slot, or one of its abilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseSlot {
    Equipment(EquipmentSlot),
    Ability(Ability),
}

impl UseSlot {
    /// How many slots there are.
    pub const COUNT: usize = EquipmentSlot::ALL.len() + Ability::ALL.len();

    /// Every slot, equipment first, in [`index`](Self::index) order.
    pub const ALL: [Self; Self::COUNT] = {
        let mut all = [Self::Equipment(EquipmentSlot::Mainhand); Self::COUNT];
        let mut index = 0;
        while index < EquipmentSlot::ALL.len() {
            all[index] = Self::Equipment(EquipmentSlot::ALL[index]);
            index += 1;
        }
        let mut ability_index = 0;
        while ability_index < Ability::ALL.len() {
            all[EquipmentSlot::ALL.len() + ability_index] =
                Self::Ability(Ability::ALL[ability_index]);
            ability_index += 1;
        }
        all
    };

    /// This slot's position in [`ALL`](Self::ALL), for arrays with one entry per slot.
    pub fn index(self) -> usize {
        match self {
            Self::Equipment(slot) => slot as usize,
            Self::Ability(ability) => EquipmentSlot::ALL.len() + ability as usize,
        }
    }
}

/// The farthest target distance at which an AI holder uses this usable.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct UseReach(pub f32);

/// A kind of item the holder must carry to use this usable, such as a heal's potion.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct RequiredItem(pub ItemKind);

impl RequiredItem {
    /// The first item of the required kind among `items`.
    pub fn find_carried(
        self,
        items: Option<&Items>,
        carried_items: &Query<&Item>,
    ) -> Option<Entity> {
        items.into_iter().flat_map(Items::iter).find(|&item| {
            carried_items
                .get(item)
                .is_ok_and(|carried| carried.kind == *self)
        })
    }
}

/// Why a character can't use a slot now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseFailure {
    /// The character has no item equipped in the slot, or lacks the ability.
    EmptySlot,
    /// The character doesn't carry the usable's [`RequiredItem`].
    MissingRequiredItem,
    /// The character is interrupted; see [`InterruptionState`].
    HolderInterrupted,
    InUse,
    OnCooldown,
    OutOfMana,
}

impl UseFailure {
    /// Whether the slot becomes usable again by waiting, rather than needing something the
    /// character lacks.
    pub fn is_temporary(self) -> bool {
        match self {
            Self::HolderInterrupted | Self::InUse | Self::OnCooldown | Self::OutOfMana => true,
            Self::EmptySlot | Self::MissingRequiredItem => false,
        }
    }
}

/// Fired at the holder when [`UseCommands::try_use`] refuses a use.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct UseFailed {
    #[event_target]
    pub holder: Entity,
    pub slot: UseSlot,
    pub reason: UseFailure,
}

/// What [`UseRead`] reads of one usable.
#[derive(QueryData)]
struct UsableState {
    entity: Entity,
    ability: Option<&'static Ability>,
    cooldown: &'static UseCooldown,
    style: &'static UseStyle,
    reach: Option<&'static UseReach>,
    mana_cost: Option<&'static ManaCost>,
    required_item: Option<&'static RequiredItem>,
    in_use: Has<InUse>,
}

/// Reads which usables a character has in each slot and whether it can use each now.
#[derive(SystemParam)]
pub struct UseRead<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    holders: Query<
        'w,
        's,
        (
            Option<&'static Mainhand>,
            Option<&'static Offhand>,
            Option<&'static Abilities>,
            Option<&'static Items>,
            Option<&'static Mana>,
            InterruptionState,
        ),
        With<Character>,
    >,
    usables: Query<'w, 's, UsableState, With<Usable>>,
    uses_in_progress: Query<'w, 's, &'static InUse>,
    carried_items: Query<'w, 's, &'static Item>,
}

impl UseRead<'_, '_> {
    /// The usable `holder` has in `slot`, if any.
    pub fn usable_in_slot(&self, holder: Entity, slot: UseSlot) -> Option<Entity> {
        let (mainhand, offhand, abilities, ..) = self.holders.get(holder).ok()?;
        match slot {
            UseSlot::Equipment(equipment_slot) => equipment_slot.equipped(mainhand, offhand),
            UseSlot::Ability(ability) => {
                abilities
                    .into_iter()
                    .flat_map(Abilities::iter)
                    .find(|&entity| {
                        self.usables
                            .get(entity)
                            .is_ok_and(|state| state.ability == Some(&ability))
                    })
            }
        }
    }

    /// How far `holder`'s usable in `slot` reaches; `None` without a [`UseReach`] or a usable.
    pub fn reach(&self, holder: Entity, slot: UseSlot) -> Option<f32> {
        self.state(holder, slot)?.reach.map(|reach| **reach)
    }

    /// Whether a use of `holder`'s usable in `slot` ends without being stopped; see
    /// [`UseStyle::ends_on_its_own`].
    pub fn ends_on_its_own(&self, holder: Entity, slot: UseSlot) -> bool {
        self.state(holder, slot)
            .is_some_and(|state| state.style.ends_on_its_own())
    }

    /// Whether `holder` is using its usable in `slot` now.
    pub fn is_using(&self, holder: Entity, slot: UseSlot) -> bool {
        self.state(holder, slot).is_some_and(|state| state.in_use)
    }

    /// The slots whose usables `holder` is using now.
    pub fn slots_in_use(&self, holder: Entity) -> impl Iterator<Item = UseSlot> + '_ {
        self.holders
            .get(holder)
            .into_iter()
            .flat_map(|(mainhand, offhand, abilities, ..)| {
                let equipped = mainhand
                    .map(Mainhand::get)
                    .into_iter()
                    .chain(offhand.map(Offhand::get));
                equipped.chain(abilities.into_iter().flat_map(Abilities::iter))
            })
            .filter_map(|entity| self.uses_in_progress.get(entity).ok())
            .map(InUse::slot)
    }

    /// The usable `holder` would use for `slot` now, or why it can't.
    pub fn can_use(&self, holder: Entity, slot: UseSlot) -> Result<Entity, UseFailure> {
        let (.., items, mana, interruption) = self
            .holders
            .get(holder)
            .map_err(|_| UseFailure::EmptySlot)?;
        let state = self.state(holder, slot).ok_or(UseFailure::EmptySlot)?;
        if let Some(&required_item) = state.required_item
            && required_item
                .find_carried(items, &self.carried_items)
                .is_none()
        {
            return Err(UseFailure::MissingRequiredItem);
        }
        if interruption.is_interrupted() {
            return Err(UseFailure::HolderInterrupted);
        }
        if state.in_use {
            return Err(UseFailure::InUse);
        }
        if !state.cooldown.is_ready(self.time.elapsed_secs()) {
            return Err(UseFailure::OnCooldown);
        }
        // A holder without a mana pool can't pay any mana cost.
        match (state.mana_cost, mana) {
            (Some(&mana_cost), Some(mana)) if !mana.can_afford(mana_cost) => {
                Err(UseFailure::OutOfMana)
            }
            (Some(_), None) => Err(UseFailure::OutOfMana),
            (Some(_), Some(_)) | (None, _) => Ok(state.entity),
        }
    }

    fn state(&self, holder: Entity, slot: UseSlot) -> Option<UsableStateItem<'_, '_>> {
        let entity = self.usable_in_slot(holder, slot)?;
        self.usables.get(entity).ok()
    }
}

/// Starts uses and stops sustained ones.
#[derive(SystemParam)]
pub struct UseCommands<'w, 's> {
    commands: Commands<'w, 's>,
    read: UseRead<'w, 's>,
}

impl<'w, 's> UseCommands<'w, 's> {
    /// The reads these commands check against.
    pub fn read(&self) -> &UseRead<'w, 's> {
        &self.read
    }

    /// Starts a use of `holder`'s usable in `slot` if [`UseRead::can_use`] allows it, or fires
    /// [`UseFailed`]. A refused use changes nothing else, so callers may retry it every frame.
    pub fn try_use(&mut self, holder: Entity, slot: UseSlot) -> Result<Entity, UseFailure> {
        let result = self.read.can_use(holder, slot);
        match result {
            Ok(entity) => {
                self.commands
                    .entity(entity)
                    .insert(InUse::new(holder, slot));
            }
            Err(reason) => {
                self.commands.trigger(UseFailed {
                    holder,
                    slot,
                    reason,
                });
            }
        }
        result
    }

    /// Stops `holder`'s use in `slot` if it is a sustained use in progress; a use that ends on its
    /// own can't be stopped. See [`UseStyle`].
    pub fn stop_sustained_use(&mut self, holder: Entity, slot: UseSlot) {
        let Some(state) = self.read.state(holder, slot) else {
            return;
        };
        if state.in_use && *state.style == UseStyle::Sustained {
            self.commands.trigger(StopUse {
                entity: state.entity,
            });
        }
    }
}
