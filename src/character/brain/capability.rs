//! What an entity can do, read from its capability components: one shared
//! trait and the table built from every capability the entity carries.

use bevy::{ecs::query::QueryData, prelude::*};

use super::ability::AbilityCapabilities;
use super::motion::FiniteMotionsReadOnly;
use super::{
    Ability, Action, Bite, Brain, BrainDefinition, Breath, Dash, Heal, Motion, MotionActionKey,
    MotionActionTable, Swoop,
};
use crate::prelude::EquipmentSlot;

/// What the decision tick needs to know about one capability, copied from the
/// component that carries it out so scoring never reaches into executor state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    /// Target distance within which the capability should be used, when it has one.
    pub reach: Option<f32>,
    /// Probability that a winning finite motion actually starts.
    pub chance: f32,
    /// Seconds before the capability may be tried again after it finishes (a
    /// finite motion also waits this out after a missed start roll).
    pub cooldown: f32,
    /// Seconds from spawn before the first use.
    pub initial_delay: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            reach: None,
            chance: 1.0,
            cooldown: 0.0,
            initial_delay: 0.0,
        }
    }
}

/// A capability component: which `MotionActionTable` entry it fills and how it is tuned.
pub(super) trait Capability {
    type Key: MotionActionKey;
    fn key(&self) -> Self::Key;
    /// What the decision pipeline needs to know to gate, score and start it.
    fn tuning(&self) -> Tuning;
}

/// What each capability an entity carries is tuned to; `None` where it has none.
/// Derefs to its table, so `capabilities[Motion::Swoop]` reads an entry directly.
#[derive(Clone, Copy, Debug, Default, Deref)]
pub(super) struct Capabilities(pub MotionActionTable<Option<Tuning>>);

/// Every capability component an entity may carry.
#[derive(QueryData)]
pub(super) struct CapabilityComponents {
    finite_motions: FiniteMotionsReadOnly,
    abilities: AbilityCapabilities,
}

/// Records one capability component, if the entity has it, under its key.
pub(super) fn record_capability<C: Capability + ?Sized>(
    capabilities: &mut Capabilities,
    component: Option<&C>,
) {
    if let Some(component) = component {
        capabilities.0[component.key()] = Some(component.tuning());
    }
}

/// Arms the opening delay of a capability added after its `Brain`, so a
/// pickup granting it still waits out the delay. Re-adding re-arms it.
fn arm_initial_delay<C: Capability + Component>(
    added: On<Add, C>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, &C)>,
) {
    let Ok((mut brain, capability)) = brains.get_mut(added.entity) else {
        return;
    };
    let now = time.elapsed_secs();
    brain.start_cooldown(capability.key(), now + capability.tuning().initial_delay);
}

/// Arms the opening delay of every capability already present when a `Brain`
/// is added.
fn arm_initial_delays(
    added: On<Add, Brain>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, CapabilityComponents)>,
) {
    let Ok((mut brain, components)) = brains.get_mut(added.entity) else {
        return;
    };
    let capabilities = build_capabilities(&components, brain.definition());
    let now = time.elapsed_secs();
    for motion in Motion::FINITE {
        if let Some(tuning) = capabilities[motion] {
            brain.start_cooldown(motion, now + tuning.initial_delay);
        }
    }
    for ability in Ability::ALL {
        if let Some(tuning) = capabilities[ability] {
            brain.start_cooldown(ability, now + tuning.initial_delay);
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(arm_initial_delays)
        .add_observer(arm_initial_delay::<Swoop>)
        .add_observer(arm_initial_delay::<Dash>)
        .add_observer(arm_initial_delay::<Bite>)
        .add_observer(arm_initial_delay::<Breath>)
        .add_observer(arm_initial_delay::<Heal>);
}

/// Collects the tuning of every capability the entity carries. Equipment is
/// always usable and shares the definition's reach.
pub(super) fn build_capabilities(
    components: &CapabilityComponentsItem,
    definition: &BrainDefinition,
) -> Capabilities {
    let mut capabilities = Capabilities::default();
    for slot in EquipmentSlot::ALL {
        capabilities.0[Action::Use(slot)] = Some(Tuning {
            reach: definition.equipment_reach,
            ..Tuning::default()
        });
    }
    components
        .finite_motions
        .record_capabilities(&mut capabilities);
    components.abilities.record_capabilities(&mut capabilities);
    capabilities
}
