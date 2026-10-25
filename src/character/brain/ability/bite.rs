use bevy::prelude::*;

use super::{AbilityCapability, Cooldown};
use crate::{
    character::{
        brain::{Ability, Action, ActionIntent, Tuning},
        vision::{TargetInfo, Targeting},
    },
    prelude::*,
};

/// Distance beyond a bite's live reach at which the decision tick arms it.
/// Decisions sample at ~10 Hz, so a dive crossing the reach could otherwise
/// fall between two samples; the applier still checks the true reach.
const DECISION_LEAD: f32 = 40.0;

/// Contact damage on a target within reach. Opt-in at spawn; absent means the
/// entity never bites. The brain decides when to bite; this component is only
/// the body's capability, so a bat bites whether it is diving or circling.
#[derive(Component, Clone, Copy, Debug)]
pub struct Bite {
    pub reach: f32,
    pub damage: f32,
    /// Fraction of the damage dealt that heals the biter.
    pub life_steal: f32,
    pub cooldown: Cooldown,
}

impl Bite {
    /// Bites for `damage` inside `reach`, at most once per `cooldown` seconds.
    pub fn new(reach: f32, damage: f32, cooldown: f32) -> Self {
        Self {
            reach,
            damage,
            life_steal: 0.0,
            cooldown: Cooldown::new(cooldown),
        }
    }

    /// Heals the biter by `fraction` of the damage each landed bite deals.
    pub fn with_life_steal(self, fraction: f32) -> Self {
        Self {
            life_steal: fraction,
            ..self
        }
    }
}

impl AbilityCapability for Bite {
    fn ability(&self) -> Ability {
        Ability::Bite
    }

    fn tuning(&self) -> Tuning {
        Tuning {
            reach: Some(self.reach + DECISION_LEAD),
            ..Tuning::default()
        }
    }
}

/// Lands a bite when the brain wants one and the target is within the true
/// reach. The decision tick only arms the intent.
pub(super) fn apply_bite(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut biters: Query<(
        Entity,
        &ActionIntent,
        &mut Bite,
        &Targeting,
        &TargetInfo,
        Option<&Health>,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, intent, mut bite, targeting, target_info, health) in &mut biters {
        if !intent.get(Action::Ability(Ability::Bite)) || !bite.cooldown.is_ready(now) {
            continue;
        }
        if !target_info.distance.is_finite() || target_info.distance > bite.reach {
            continue;
        }
        commands.trigger(AttemptDamage {
            entity: targeting.0,
            damage: Damage::Single(bite.damage),
            source_item: None,
            attacker: Some(entity),
            direction: Some(target_info.direction),
            ..default()
        });
        // Skip the heal at full health so the overlay does not pop a zero.
        if bite.life_steal > 0.0 && health.is_some_and(|health| health.hp < health.max_hp) {
            commands.trigger(AttemptHeal {
                entity,
                amount: bite.damage * bite.life_steal,
            });
        }
        bite.cooldown.start(now);
    }
}
