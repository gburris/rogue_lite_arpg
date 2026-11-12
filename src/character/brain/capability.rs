//! What a character can do and when: the [`CapabilityProfile`] a decision reads for each motion
//! and action, the capability components behind them, and the [`Cooldowns`] that pace them.

use bevy::ecs::query::QueryData;

use super::action::{Ability, Action, Bite, Breath, Heal};
use super::motion::{Dash, Motion, Orbit, Swoop, Wander};

/// The reach, start chance and cooldown a decision reads for one motion or action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CapabilityProfile {
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

/// Every motion capability component a character may carry.
#[derive(QueryData)]
pub(super) struct MotionCapabilities {
    dash: Option<&'static Dash>,
    swoop: Option<&'static Swoop>,
    orbit: Option<&'static Orbit>,
    wander: Option<&'static Wander>,
}

impl MotionCapabilitiesItem<'_, '_> {
    /// The profile of the capability that carries out `motion`; `None` when the character lacks
    /// it. Motions steered without a component are always owned.
    pub(super) fn capability_profile(&self, motion: Motion) -> Option<CapabilityProfile> {
        match motion {
            // Their components carry no decision settings, only ownership.
            Motion::Wander => self.wander.map(|_| CapabilityProfile::default()),
            Motion::Orbit(_) => self.orbit.map(|_| CapabilityProfile::default()),
            Motion::Dash => self.dash.map(Dash::capability_profile),
            Motion::Swoop => self.swoop.map(Swoop::capability_profile),
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle => Some(CapabilityProfile::default()),
        }
    }
}

/// Every ability capability component a character may carry.
#[derive(QueryData)]
pub(super) struct AbilityCapabilities {
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    heal: Option<&'static Heal>,
}

impl AbilityCapabilitiesItem<'_, '_> {
    /// The profile of the component that carries out `ability`; `None` when the character
    /// lacks it.
    pub(super) fn capability_profile(&self, ability: Ability) -> Option<CapabilityProfile> {
        match ability {
            Ability::Heal => self.heal.copied().map(Heal::capability_profile),
            Ability::Breathe => self.breath.copied().map(Breath::capability_profile),
            Ability::Bite => self.bite.copied().map(Bite::capability_profile),
        }
    }
}

/// A motion or action with its own entry in [`Cooldowns`].
pub(super) trait CooldownKey: Copy {
    fn cooldown_index(self) -> usize;
}

/// Motions come first in [`Cooldowns`].
impl CooldownKey for Motion {
    fn cooldown_index(self) -> usize {
        self.index()
    }
}

/// Actions follow the motions in [`Cooldowns`].
impl CooldownKey for Action {
    fn cooldown_index(self) -> usize {
        Motion::COUNT + self.index()
    }
}

impl CooldownKey for Ability {
    fn cooldown_index(self) -> usize {
        Action::Ability(self).cooldown_index()
    }
}

/// The virtual time at which each motion and action is off cooldown.
#[derive(Clone, Copy, Debug)]
pub(super) struct Cooldowns([f32; Motion::COUNT + Action::COUNT]);

impl Default for Cooldowns {
    fn default() -> Self {
        Self([0.0; Motion::COUNT + Action::COUNT])
    }
}

impl Cooldowns {
    pub(super) fn is_ready(&self, key: impl CooldownKey, now: f32) -> bool {
        now >= self.0[key.cooldown_index()]
    }

    pub(super) fn start(&mut self, key: impl CooldownKey, ready_at: f32) {
        self.0[key.cooldown_index()] = ready_at;
    }
}
