use bevy::prelude::*;

use crate::character::brain::perception::Targeting;
use crate::character::brain::{Ability, Action, Brain, Capability, CapabilityProfile};
use crate::character::vision::TargetInfo;
use crate::prelude::*;

/// Contact damage on a target within reach.
#[derive(Component, Clone, Copy, Debug)]
pub struct Bite {
    pub reach: f32,
    pub damage: f32,
    pub life_steal: f32,
    pub cooldown: f32,
}

impl Bite {
    /// Bites for `damage` inside `reach`, at most once per `cooldown` seconds.
    pub fn new(reach: f32, damage: f32, cooldown: f32) -> Self {
        Self {
            reach,
            damage,
            life_steal: 0.0,
            cooldown,
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

impl Capability for Bite {
    type Key = Ability;

    fn key(&self) -> Ability {
        Ability::Bite
    }

    fn capability_profile(&self) -> CapabilityProfile {
        CapabilityProfile {
            reach: Some(self.reach),
            cooldown: self.cooldown,
            ..CapabilityProfile::default()
        }
    }
}

/// Bites a target within the true reach; contact needs no line of sight.
pub(super) fn apply_bite(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    biters: Query<(
        Entity,
        &Brain,
        &Bite,
        &Targeting,
        &TargetInfo,
        Option<&Health>,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, brain, bite, targeting, target_info, health) in &biters {
        let Some(biting) = brain.held_action(Action::Ability(Ability::Bite)) else {
            continue;
        };
        let in_reach = target_info.distance.is_finite() && target_info.distance <= bite.reach;
        if !(brain.is_cooldown_ready(Ability::Bite, now) && in_reach) {
            biting.report_not_started(&mut commands, entity);
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
        biting.report_completed(&mut commands, entity);
    }
}
