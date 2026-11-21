use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ActionSource;
use super::in_progress::{ActionInProgress, StopAction};
use crate::prelude::*;

/// Why a character can't use an action source now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionFailure {
    /// The character has no item equipped under the key, or lacks the ability.
    NoSource,
    /// The character doesn't carry the action source's [`RequiredItem`].
    MissingRequiredItem,
    /// The character is interrupted; see [`InterruptionState`].
    HolderInterrupted,
    /// A use of the action source is already under way.
    InProgress,
    /// The action source's [`Cooldown`] hasn't run out.
    OnCooldown,
    /// The character can't pay the action source's [`ManaCost`].
    OutOfMana,
}

impl ActionFailure {
    /// Whether the action source becomes usable again by waiting, rather than needing something
    /// the character lacks.
    pub fn is_temporary(self) -> bool {
        match self {
            Self::HolderInterrupted | Self::InProgress | Self::OnCooldown | Self::OutOfMana => true,
            Self::NoSource | Self::MissingRequiredItem => false,
        }
    }
}

/// Fired at the holder when [`ActionCommands::try_use`] refuses a use.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFailed {
    #[event_target]
    pub holder: Entity,
    pub key: ActionKey,
    pub reason: ActionFailure,
}

/// What [`ActionStatus`] reads of one action source.
#[derive(QueryData)]
struct ActionSourceData {
    entity: Entity,
    ability: Option<&'static Ability>,
    cooldown: &'static Cooldown,
    style: &'static ActionUseStyle,
    reach: Option<&'static ActionReach>,
    is_attack: Has<Attack>,
    mana_cost: Option<&'static ManaCost>,
    required_item: Option<&'static RequiredItem>,
    in_progress: Option<&'static ActionInProgress>,
}

/// What [`ActionStatus`] reads of a character holding action sources.
#[derive(QueryData)]
struct ActionHolder {
    mainhand: Option<&'static Mainhand>,
    offhand: Option<&'static Offhand>,
    abilities: Option<&'static Abilities>,
    mana: Option<&'static Mana>,
    interruption: InterruptionState,
}

impl ActionHolderItem<'_, '_> {
    fn equipped_sources(&self) -> impl Iterator<Item = Entity> {
        let mainhand = self.mainhand.map(Mainhand::get);
        mainhand.into_iter().chain(self.offhand.map(Offhand::get))
    }

    fn ability_sources(&self) -> impl Iterator<Item = Entity> {
        self.abilities.into_iter().flat_map(Abilities::iter)
    }
}

/// Reads which action sources a character has and whether it can use each now.
#[derive(SystemParam)]
pub struct ActionStatus<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    holders: Query<'w, 's, ActionHolder, With<Character>>,
    sources: Query<'w, 's, ActionSourceData, With<ActionSource>>,
    carried_items: CarriedItems<'w, 's>,
}

impl ActionStatus<'_, '_> {
    /// The action source `key` names on `holder`, if any.
    pub fn source_for_key(&self, holder: Entity, key: ActionKey) -> Option<Entity> {
        let holder = self.holders.get(holder).ok()?;
        match key {
            ActionKey::Equipped(equipment_slot) => {
                equipment_slot.equipped(holder.mainhand, holder.offhand)
            }
            ActionKey::Ability(ability) => holder.ability_sources().find(|&entity| {
                self.sources
                    .get(entity)
                    .is_ok_and(|source| source.ability == Some(&ability))
            }),
        }
    }

    /// How far `holder`'s action source named by `key` reaches; `None` without an [`ActionReach`]
    /// or an action source.
    pub fn reach(&self, holder: Entity, key: ActionKey) -> Option<f32> {
        self.source_data(holder, key)?
            .reach
            .map(|&ActionReach(reach)| reach)
    }

    /// Whether a use of `holder`'s action source named by `key` ends without being stopped; see
    /// [`ActionUseStyle::ends_on_its_own`].
    pub fn ends_on_its_own(&self, holder: Entity, key: ActionKey) -> bool {
        self.source_data(holder, key)
            .is_some_and(|source| source.style.ends_on_its_own())
    }

    /// Whether a use of `holder`'s action source named by `key` is an attack; see [`Attack`].
    pub fn is_attack(&self, holder: Entity, key: ActionKey) -> bool {
        self.source_data(holder, key)
            .is_some_and(|source| source.is_attack)
    }

    /// Whether `holder` is using its action source named by `key` now.
    pub fn is_using(&self, holder: Entity, key: ActionKey) -> bool {
        self.source_data(holder, key)
            .is_some_and(|source| source.in_progress.is_some())
    }

    /// Whether `holder` is using an item whose use plays out and ends itself, such as a swing; see
    /// [`ActionUseStyle::Timed`].
    pub fn is_using_timed_item(&self, holder: Entity) -> bool {
        self.is_using_any_matching(holder, |source| {
            source.ability.is_none() && matches!(source.style, ActionUseStyle::Timed { .. })
        })
    }

    /// Whether `holder` is using an action source marked [`Attack`] now.
    pub fn is_attacking(&self, holder: Entity) -> bool {
        self.is_using_any_matching(holder, |source| source.is_attack)
    }

    /// Whether `holder` is using any of its action sources now.
    pub fn is_using_any(&self, holder: Entity) -> bool {
        self.is_using_any_matching(holder, |_| true)
    }

    /// The action source `holder` would use for `key` now, or why it can't.
    pub fn can_use(&self, holder: Entity, key: ActionKey) -> Result<Entity, ActionFailure> {
        let source = self
            .source_data(holder, key)
            .ok_or(ActionFailure::NoSource)?;
        if let Some(&RequiredItem(kind)) = source.required_item
            && self.carried_items.first_of_kind(holder, kind).is_none()
        {
            return Err(ActionFailure::MissingRequiredItem);
        }
        let holder = self
            .holders
            .get(holder)
            .map_err(|_| ActionFailure::NoSource)?;
        if holder.interruption.is_interrupted() {
            return Err(ActionFailure::HolderInterrupted);
        }
        if source.in_progress.is_some() {
            return Err(ActionFailure::InProgress);
        }
        if !source.cooldown.is_ready(self.time.elapsed_secs()) {
            return Err(ActionFailure::OnCooldown);
        }
        if let Some(&mana_cost) = source.mana_cost
            && !mana_cost.is_affordable_by(holder.mana)
        {
            return Err(ActionFailure::OutOfMana);
        }
        Ok(source.entity)
    }

    fn is_using_any_matching(
        &self,
        holder: Entity,
        predicate: impl Fn(&ActionSourceDataItem) -> bool,
    ) -> bool {
        let Ok(holder) = self.holders.get(holder) else {
            return false;
        };
        holder
            .equipped_sources()
            .chain(holder.ability_sources())
            .any(|entity| {
                self.sources
                    .get(entity)
                    .is_ok_and(|source| source.in_progress.is_some() && predicate(&source))
            })
    }

    fn source_data(&self, holder: Entity, key: ActionKey) -> Option<ActionSourceDataItem<'_, '_>> {
        let entity = self.source_for_key(holder, key)?;
        self.sources.get(entity).ok()
    }
}

/// Starts uses and stops sustained ones.
#[derive(SystemParam)]
pub struct ActionCommands<'w, 's> {
    commands: Commands<'w, 's>,
    status: ActionStatus<'w, 's>,
}

impl<'w, 's> ActionCommands<'w, 's> {
    /// The status these commands check against.
    pub fn status(&self) -> &ActionStatus<'w, 's> {
        &self.status
    }

    /// Starts a use of `holder`'s action source named by `key` if [`ActionStatus::can_use`] allows
    /// it, or fires [`ActionFailed`]. A refused use changes nothing else, so callers may retry
    /// it every frame.
    pub fn try_use(&mut self, holder: Entity, key: ActionKey) -> Result<Entity, ActionFailure> {
        let result = self.status.can_use(holder, key);
        match result {
            Ok(entity) => {
                self.commands
                    .entity(entity)
                    .insert(ActionInProgress::new(holder, key));
            }
            Err(reason) => {
                self.commands.trigger(ActionFailed {
                    holder,
                    key,
                    reason,
                });
            }
        }
        result
    }

    /// Stops `holder`'s use of the action source named by `key` if it is a sustained use in
    /// progress; a use that ends on its own can't be stopped. See [`ActionUseStyle`].
    pub fn stop_sustained_use(&mut self, holder: Entity, key: ActionKey) {
        let Some(source) = self.status.source_data(holder, key) else {
            return;
        };
        if source.in_progress.is_some() && *source.style == ActionUseStyle::Sustained {
            self.commands.trigger(StopAction {
                source: source.entity,
            });
        }
    }
}

/// Names an [`ActionSource`] of a character: the item equipped in a slot, or one of its abilities
/// by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionKey {
    Equipped(EquipmentSlot),
    Ability(Ability),
}

impl ActionKey {
    /// Every key, equipment first.
    pub fn all() -> impl Iterator<Item = Self> {
        let equipment = EquipmentSlot::ALL.into_iter().map(Self::Equipped);
        equipment.chain(Ability::ALL.into_iter().map(Self::Ability))
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

/// How a use ends once it takes effect.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ActionUseStyle {
    /// Ends the moment it takes effect, like a heal or a tome's spell.
    #[default]
    Instant,
    /// Plays out for `seconds` after it takes effect, then ends, like a sword swing or a staff
    /// cast.
    Timed { seconds: f32 },
    /// Lasts until it is stopped, like a shield block.
    Sustained,
}

impl ActionUseStyle {
    /// Whether a use in this style ends without being stopped.
    pub fn ends_on_its_own(self) -> bool {
        match self {
            Self::Instant | Self::Timed { .. } => true,
            Self::Sustained => false,
        }
    }
}

/// How long each use winds up before it takes effect, a window for a hit to interrupt it. Without
/// it, a use takes effect on the frame it starts.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionWindup {
    pub seconds: f32,
}

/// Lights the holder in this color while a use winds up.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionGlow(pub Color);
