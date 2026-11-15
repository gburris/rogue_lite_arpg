//! What a character can do and when: the [`CapabilityProfile`] a decision reads for each motion
//! and action, the motion capability components behind them, and the [`Cooldowns`] that pace
//! motions.

use bevy::ecs::query::QueryData;

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

/// The virtual time at which each motion is off cooldown.
#[derive(Clone, Copy, Debug)]
pub(super) struct Cooldowns([f32; Motion::COUNT]);

impl Default for Cooldowns {
    fn default() -> Self {
        Self([0.0; Motion::COUNT])
    }
}

impl Cooldowns {
    pub(super) fn is_ready(&self, motion: Motion, now: f32) -> bool {
        now >= self.0[motion.index()]
    }

    pub(super) fn start(&mut self, motion: Motion, ready_at: f32) {
        self.0[motion.index()] = ready_at;
    }
}
