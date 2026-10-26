//! Abilities a brain can ask for besides moving or using equipment. Each one
//! is an opt-in capability component plus an applier that reads the brain's
//! held action ([`Brain::action`](super::Brain::action)), re-checks the live world,
//! and does the work. Appliers never change the brain's motion. A
//! defeated body loses its brain, so appliers never see the dead.
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

use super::{Ability, Brain, Cooldown, Tuning};
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

/// Arms each ability's opening delay when a brain is added, so a first use
/// can wait from spawn rather than from the first wanted use.
fn arm_initial_delays(
    added: On<Add, Brain>,
    time: Res<Time<Virtual>>,
    mut brains: Query<(&mut Brain, AbilityCapabilities)>,
) {
    let Ok((mut brain, abilities)) = brains.get_mut(added.entity) else {
        return;
    };
    let now = time.elapsed_secs();
    for (ability, tuning) in Ability::ALL.into_iter().zip(abilities.tuning()) {
        if let Some(tuning) = tuning {
            brain.start_cooldown(Cooldown::Ability(ability), now + tuning.initial_delay);
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(arm_initial_delays).add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(AiSet::Execute),
    );
}
