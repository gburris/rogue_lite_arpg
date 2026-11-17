use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ActionSource;
use super::in_progress::{ActionCooldown, ActionInProgress, ActionUseStyle, StopAction};
use crate::prelude::*;

/// Where a character finds an [`ActionSource`]: the item equipped in a slot, or one of its
/// abilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionSlot {
    Equipment(EquipmentSlot),
    Ability(Ability),
}

impl ActionSlot {
    /// How many slots there are.
    pub const COUNT: usize = EquipmentSlot::ALL.len() + Ability::ALL.len();

    /// Every slot, equipment first, in [`index`](Self::index) order.
    pub fn all() -> impl Iterator<Item = Self> {
        let equipment = EquipmentSlot::ALL.into_iter().map(Self::Equipment);
        equipment.chain(Ability::ALL.into_iter().map(Self::Ability))
    }

    /// This slot's position in [`all`](Self::all), for arrays with one entry per slot.
    pub fn index(self) -> usize {
        match self {
            Self::Equipment(slot) => slot as usize,
            Self::Ability(ability) => EquipmentSlot::ALL.len() + ability as usize,
        }
    }
}

/// The farthest target distance at which an AI holder uses this action source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct ActionReach(pub f32);

/// Marks an action source whose use is an attack, like a swing, a cast or a bite.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Attack;

/// A kind of item the holder must carry to use this action source, such as a heal's potion.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct RequiredItem(pub ItemKind);

/// Why a character can't use a slot now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionFailure {
    /// The character has no item equipped in the slot, or lacks the ability.
    EmptySlot,
    /// The character doesn't carry the action source's [`RequiredItem`].
    MissingRequiredItem,
    /// The character is interrupted; see [`InterruptionState`].
    HolderInterrupted,
    /// A use of the action source is already under way.
    InProgress,
    /// The action source's [`ActionCooldown`] hasn't run out.
    OnCooldown,
    /// The character can't pay the action source's [`ManaCost`].
    OutOfMana,
}

impl ActionFailure {
    /// Whether the slot becomes usable again by waiting, rather than needing something the
    /// character lacks.
    pub fn is_temporary(self) -> bool {
        match self {
            Self::HolderInterrupted | Self::InProgress | Self::OnCooldown | Self::OutOfMana => true,
            Self::EmptySlot | Self::MissingRequiredItem => false,
        }
    }
}

/// Fired at the holder when [`ActionCommands::try_use`] refuses a use.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFailed {
    #[event_target]
    pub holder: Entity,
    pub slot: ActionSlot,
    pub reason: ActionFailure,
}

/// What [`ActionRead`] reads of one action source.
#[derive(QueryData)]
struct ActionSourceState {
    entity: Entity,
    ability: Option<&'static Ability>,
    cooldown: &'static ActionCooldown,
    style: &'static ActionUseStyle,
    reach: Option<&'static ActionReach>,
    is_attack: Has<Attack>,
    mana_cost: Option<&'static ManaCost>,
    required_item: Option<&'static RequiredItem>,
    in_progress: Option<&'static ActionInProgress>,
}

/// What [`ActionRead`] reads of a character holding action sources.
#[derive(QueryData)]
struct ActionHolder {
    mainhand: Option<&'static Mainhand>,
    offhand: Option<&'static Offhand>,
    abilities: Option<&'static Abilities>,
    mana: Option<&'static Mana>,
    interruption: InterruptionState,
}

impl ActionHolderItem<'_, '_> {
    fn equipped_items(&self) -> impl Iterator<Item = Entity> {
        let mainhand = self.mainhand.map(Mainhand::get);
        mainhand.into_iter().chain(self.offhand.map(Offhand::get))
    }

    fn ability_sources(&self) -> impl Iterator<Item = Entity> {
        self.abilities.into_iter().flat_map(Abilities::iter)
    }
}

/// Reads which sources a character has in each slot and whether it can use each now.
#[derive(SystemParam)]
pub struct ActionRead<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    holders: Query<'w, 's, ActionHolder, With<Character>>,
    sources: Query<'w, 's, ActionSourceState, With<ActionSource>>,
    carried_items: CarriedItems<'w, 's>,
}

impl ActionRead<'_, '_> {
    /// The action source `holder` has in `slot`, if any.
    pub fn source_in_slot(&self, holder: Entity, slot: ActionSlot) -> Option<Entity> {
        let holder = self.holders.get(holder).ok()?;
        match slot {
            ActionSlot::Equipment(equipment_slot) => {
                equipment_slot.equipped(holder.mainhand, holder.offhand)
            }
            ActionSlot::Ability(ability) => holder.ability_sources().find(|&entity| {
                self.sources
                    .get(entity)
                    .is_ok_and(|state| state.ability == Some(&ability))
            }),
        }
    }

    /// How far `holder`'s action source in `slot` reaches; `None` without an [`ActionReach`] or an
    /// action source.
    pub fn reach(&self, holder: Entity, slot: ActionSlot) -> Option<f32> {
        self.state(holder, slot)?
            .reach
            .map(|&ActionReach(reach)| reach)
    }

    /// Whether a use of `holder`'s action source in `slot` ends without being stopped; see
    /// [`ActionUseStyle::ends_on_its_own`].
    pub fn ends_on_its_own(&self, holder: Entity, slot: ActionSlot) -> bool {
        self.state(holder, slot)
            .is_some_and(|state| state.style.ends_on_its_own())
    }

    /// Whether a use of `holder`'s action source in `slot` is an attack; see [`Attack`].
    pub fn is_attack(&self, holder: Entity, slot: ActionSlot) -> bool {
        self.state(holder, slot)
            .is_some_and(|state| state.is_attack)
    }

    /// Whether `holder` is using its action source in `slot` now.
    pub fn is_using(&self, holder: Entity, slot: ActionSlot) -> bool {
        self.state(holder, slot)
            .is_some_and(|state| state.in_progress.is_some())
    }

    /// Whether `holder` is using an item whose use plays out and ends itself, such as a swing; see
    /// [`ActionUseStyle::Timed`].
    pub fn is_using_timed_item(&self, holder: Entity) -> bool {
        let Ok(holder) = self.holders.get(holder) else {
            return false;
        };
        holder
            .equipped_items()
            .filter_map(|item| self.sources.get(item).ok())
            .any(|state| state.in_progress.is_some() && *state.style == ActionUseStyle::Timed)
    }

    /// Whether `holder` is using any of its action sources now.
    pub fn is_using_any(&self, holder: Entity) -> bool {
        let Ok(holder) = self.holders.get(holder) else {
            return false;
        };
        holder
            .equipped_items()
            .chain(holder.ability_sources())
            .any(|entity| {
                self.sources
                    .get(entity)
                    .is_ok_and(|state| state.in_progress.is_some())
            })
    }

    /// The action source `holder` would use for `slot` now, or why it can't.
    pub fn can_use(&self, holder: Entity, slot: ActionSlot) -> Result<Entity, ActionFailure> {
        let state = self.state(holder, slot).ok_or(ActionFailure::EmptySlot)?;
        if let Some(&RequiredItem(kind)) = state.required_item
            && self.carried_items.first_of_kind(holder, kind).is_none()
        {
            return Err(ActionFailure::MissingRequiredItem);
        }
        let holder = self
            .holders
            .get(holder)
            .map_err(|_| ActionFailure::EmptySlot)?;
        if holder.interruption.is_interrupted() {
            return Err(ActionFailure::HolderInterrupted);
        }
        if state.in_progress.is_some() {
            return Err(ActionFailure::InProgress);
        }
        if !state.cooldown.is_ready(self.time.elapsed_secs()) {
            return Err(ActionFailure::OnCooldown);
        }
        if let Some(&mana_cost) = state.mana_cost
            && !mana_cost.is_affordable_by(holder.mana)
        {
            return Err(ActionFailure::OutOfMana);
        }
        Ok(state.entity)
    }

    fn state(&self, holder: Entity, slot: ActionSlot) -> Option<ActionSourceStateItem<'_, '_>> {
        let entity = self.source_in_slot(holder, slot)?;
        self.sources.get(entity).ok()
    }
}

/// Starts uses and stops sustained ones.
#[derive(SystemParam)]
pub struct ActionCommands<'w, 's> {
    commands: Commands<'w, 's>,
    read: ActionRead<'w, 's>,
}

impl<'w, 's> ActionCommands<'w, 's> {
    /// The reads these commands check against.
    pub fn read(&self) -> &ActionRead<'w, 's> {
        &self.read
    }

    /// Starts a use of `holder`'s action source in `slot` if [`ActionRead::can_use`] allows it, or
    /// fires [`ActionFailed`]. A refused use changes nothing else, so callers may retry it
    /// every frame.
    pub fn try_use(&mut self, holder: Entity, slot: ActionSlot) -> Result<Entity, ActionFailure> {
        let result = self.read.can_use(holder, slot);
        match result {
            Ok(entity) => {
                self.commands
                    .entity(entity)
                    .insert(ActionInProgress::new(holder, slot));
            }
            Err(reason) => {
                self.commands.trigger(ActionFailed {
                    holder,
                    slot,
                    reason,
                });
            }
        }
        result
    }

    /// Stops `holder`'s use in `slot` if it is a sustained use in progress; a use that ends on its
    /// own can't be stopped. See [`ActionUseStyle`].
    pub fn stop_sustained_use(&mut self, holder: Entity, slot: ActionSlot) {
        let Some(state) = self.read.state(holder, slot) else {
            return;
        };
        if state.in_progress.is_some() && *state.style == ActionUseStyle::Sustained {
            self.commands.trigger(StopAction {
                source: state.entity,
            });
        }
    }
}
