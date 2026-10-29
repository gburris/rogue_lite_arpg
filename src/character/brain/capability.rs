//! What an entity can do, read from its capability components: one shared
//! trait, the per-motion-and-action table and its keys, and the tables built
//! from every capability the entity carries.

use bevy::{ecs::query::QueryData, prelude::*};

use super::action::{Ability, AbilityCapabilities, Action};
use super::motion::{Dash, Motion, Orbit, Swoop, Wander};
use crate::prelude::{EquipmentRead, EquipmentSlot};

/// A type with its own entry in a [`MotionActionTable`].
pub trait MotionActionKey: Copy {
    fn table_index(self) -> usize;
}

/// Motions come first in a [`MotionActionTable`].
impl MotionActionKey for Motion {
    fn table_index(self) -> usize {
        self.index()
    }
}

/// Actions follow the motions in a [`MotionActionTable`].
impl MotionActionKey for Action {
    fn table_index(self) -> usize {
        Motion::COUNT + self.index()
    }
}

impl MotionActionKey for Ability {
    fn table_index(self) -> usize {
        Action::Ability(self).table_index()
    }
}

/// One value per motion and per action, indexed by any [`MotionActionKey`].
#[derive(Clone, Copy, Debug)]
pub struct MotionActionTable<T>([T; Motion::COUNT + Action::COUNT]);

impl<T: Copy> MotionActionTable<T> {
    pub fn splat(value: T) -> Self {
        Self([value; Motion::COUNT + Action::COUNT])
    }
}

impl<T: Default> Default for MotionActionTable<T> {
    fn default() -> Self {
        Self(std::array::from_fn(|_| T::default()))
    }
}

impl<T, K: MotionActionKey> std::ops::Index<K> for MotionActionTable<T> {
    type Output = T;

    fn index(&self, key: K) -> &T {
        &self.0[key.table_index()]
    }
}

impl<T, K: MotionActionKey> std::ops::IndexMut<K> for MotionActionTable<T> {
    fn index_mut(&mut self, key: K) -> &mut T {
        &mut self.0[key.table_index()]
    }
}

/// Which motions and actions may start right now.
#[derive(Clone, Copy, Debug, Deref)]
pub struct AvailableMotionActionTable(pub MotionActionTable<bool>);

/// Which motions and actions the world lets run right now: usable equipment
/// and a carried potion for `Heal`. One input to [`AvailableMotionActionTable`].
#[derive(Clone, Copy, Debug, Deref)]
pub struct UseEligibilityTable(pub MotionActionTable<bool>);

impl Default for UseEligibilityTable {
    fn default() -> Self {
        Self(MotionActionTable::splat(true))
    }
}

/// What the decision tick needs to know about one capability, copied from the
/// component that carries it out so scoring never reaches into its running state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CapabilityProfile {
    pub reach: Option<f32>,
    /// Probability that a winning finite motion actually starts.
    pub chance: f32,
    /// Seconds to wait after a finish or a missed start roll.
    pub cooldown: f32,
}

impl Default for CapabilityProfile {
    fn default() -> Self {
        Self {
            reach: None,
            chance: 1.0,
            cooldown: 0.0,
        }
    }
}

/// A motion or action that only runs when a capability fills its entry.
#[derive(Clone, Copy, Debug)]
pub(super) enum CapabilityKey {
    Motion(Motion),
    Action(Action),
}

impl MotionActionKey for CapabilityKey {
    fn table_index(self) -> usize {
        match self {
            Self::Motion(motion) => motion.table_index(),
            Self::Action(action) => action.table_index(),
        }
    }
}

/// Every key a capability can fill: each motion that needs a component to
/// steer, each equipment slot's use and each ability. `Action::Idle` needs none.
pub(super) fn capability_keys() -> impl Iterator<Item = CapabilityKey> {
    let motions = Motion::ALL.iter().copied().filter(|motion| match motion {
        Motion::Wander | Motion::Orbit(_) | Motion::Dash | Motion::Swoop => true,
        Motion::Chase
        | Motion::ReturnToHome
        | Motion::Flee
        | Motion::KeepDistance(_)
        | Motion::Idle => false,
    });
    let actions = EquipmentSlot::ALL
        .into_iter()
        .map(Action::Use)
        .chain(Ability::ALL.into_iter().map(Action::Ability));
    motions
        .map(CapabilityKey::Motion)
        .chain(actions.map(CapabilityKey::Action))
}

/// A capability component: which `MotionActionTable` entry it fills and its [`CapabilityProfile`].
pub(super) trait Capability {
    type Key: MotionActionKey;
    fn key(&self) -> Self::Key;
    fn capability_profile(&self) -> CapabilityProfile;
}

/// Each carried capability's profile; `None` where the entity has none.
#[derive(Clone, Copy, Debug, Default, Deref)]
pub(super) struct CapabilityProfiles(pub MotionActionTable<Option<CapabilityProfile>>);

/// Every capability component an entity may carry.
#[derive(QueryData)]
pub(super) struct CapabilityComponents {
    entity: Entity,
    dash: Option<&'static Dash>,
    swoop: Option<&'static Swoop>,
    orbit: Option<&'static Orbit>,
    wander: Option<&'static Wander>,
    abilities: AbilityCapabilities,
}

pub(super) fn record_capability_profile<C: Capability>(
    capability_profiles: &mut CapabilityProfiles,
    component: Option<&C>,
) {
    if let Some(component) = component {
        capability_profiles.0[component.key()] = Some(component.capability_profile());
    }
}

/// Collects the profile of every capability the entity carries. Each equipped
/// item is a capability with the item's own reach.
pub(super) fn build_capability_profiles(
    components: &CapabilityComponentsItem,
    equipment: &EquipmentRead,
) -> CapabilityProfiles {
    let mut capability_profiles = CapabilityProfiles::default();
    for slot in EquipmentSlot::ALL {
        capability_profiles.0[Action::Use(slot)] = equipment
            .equippable_in_slot(components.entity, slot)
            .map(|equippable| CapabilityProfile {
                reach: equippable.reach,
                ..CapabilityProfile::default()
            });
    }
    record_capability_profile(&mut capability_profiles, components.dash);
    record_capability_profile(&mut capability_profiles, components.swoop);
    record_capability_profile(&mut capability_profiles, components.orbit);
    record_capability_profile(&mut capability_profiles, components.wander);
    components
        .abilities
        .record_capability_profiles(&mut capability_profiles);
    capability_profiles
}
