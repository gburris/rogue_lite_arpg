//! Abilities a brain can ask for besides moving or using equipment. Each one
//! is an opt-in capability component plus an applier that reads the brain's
//! [`ActionIntent`](super::ActionIntent), re-checks the live world,
//! and does the work. Appliers never change the brain's motion. A
//! defeated body loses its intent, so appliers never see the dead.
//!
//! Capability components implement [`AbilityCapability`] and are listed once,
//! in [`AbilityCapabilities`], which is how the brain's senses learn what an
//! entity can do without naming each component.

mod bite;
mod breath;
mod heal;

use bevy::{ecs::query::QueryData, prelude::*};

pub use bite::Bite;
pub use breath::Breath;
pub use heal::Heal;

use super::{Ability, Tuning};
use crate::character::AiSet;

/// A capability component that carries out one [`Ability`].
pub trait AbilityCapability {
    fn ability(&self) -> Ability;
    /// What the decision tick needs to know to score this ability.
    fn tuning(&self) -> Tuning;
}

/// Every ability capability an entity may carry. The one place to list a new one.
#[derive(QueryData)]
pub struct AbilityCapabilities {
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    heal: Option<&'static Heal>,
}

impl AbilityCapabilitiesItem<'_, '_> {
    /// Tuning of every ability this entity can carry out, by ability index.
    pub fn tuning(&self) -> [Option<Tuning>; Ability::ALL.len()] {
        let capabilities: [Option<&dyn AbilityCapability>; 3] = [
            self.bite
                .map(|capability| capability as &dyn AbilityCapability),
            self.breath
                .map(|capability| capability as &dyn AbilityCapability),
            self.heal
                .map(|capability| capability as &dyn AbilityCapability),
        ];
        let mut tuning = [None; Ability::ALL.len()];
        for capability in capabilities.into_iter().flatten() {
            tuning[capability.ability() as usize] = Some(capability.tuning());
        }
        tuning
    }
}

/// Spacing between uses of one ability, in virtual seconds.
#[derive(Clone, Copy, Debug)]
pub struct Cooldown {
    pub seconds: f32,
    /// `None` until first checked, for a cooldown that starts on its own.
    ready_at: Option<f32>,
}

impl Cooldown {
    /// Ready at once; each use waits `seconds`.
    pub fn new(seconds: f32) -> Self {
        Self {
            seconds,
            ready_at: Some(0.0),
        }
    }

    /// Like [`Cooldown::new`], but the first use also waits `seconds` from
    /// the first check.
    pub fn delayed(seconds: f32) -> Self {
        Self {
            seconds,
            ready_at: None,
        }
    }

    /// Whether the ability may be used at `now`.
    pub fn is_ready(&mut self, now: f32) -> bool {
        now >= *self.ready_at.get_or_insert(now + self.seconds)
    }

    /// Records a use at `now`.
    pub fn start(&mut self, now: f32) {
        self.ready_at = Some(now + self.seconds);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(AiSet::Execute),
    );
}
