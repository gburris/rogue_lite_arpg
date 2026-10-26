use bevy::prelude::*;

use crate::character::brain::Capability;
use crate::{
    character::{
        brain::{Ability, Action, ActionFinished, Brain, CAST_MAX_RANGE, Tuning},
        vision::{TargetInfo, Targeting},
    },
    prelude::*,
};

/// An innate projectile attack that needs no weapon. The projectile template
/// rides on the entity as a `Projectiles` relation. Opt-in at spawn; absent
/// means the entity never breathes.
#[derive(Component, Debug)]
pub struct Breath {
    pub cooldown: f32,
    /// Seconds from spawn before the first breath.
    pub initial_delay: f32,
    /// Farthest target distance at which the brain breathes.
    pub reach: f32,
}

impl Breath {
    /// Breathes at most once per `period` seconds within cast range; the
    /// first breath waits one period.
    pub fn new(period: f32) -> Self {
        Self {
            cooldown: period,
            initial_delay: period,
            reach: CAST_MAX_RANGE,
        }
    }
}

impl Capability for Breath {
    type Key = Ability;

    fn key(&self) -> Ability {
        Ability::Breathe
    }

    fn tuning(&self) -> Tuning {
        Tuning {
            reach: Some(self.reach),
            cooldown: self.cooldown,
            initial_delay: self.initial_delay,
            ..Tuning::default()
        }
    }
}

/// Launches the innate projectile whenever the brain wants to breathe at a
/// sighted target.
pub(super) fn apply_breath(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    breathers: Query<(
        Entity,
        &Brain,
        Has<Targeting>,
        &TargetInfo,
        &Transform,
        &CombatSide,
        &Projectiles,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, brain, has_target, target_info, transform, side, templates) in &breathers {
        let ready = brain.is_cooldown_ready(Ability::Breathe, now);
        let wanted = brain.action() == Some(Action::Ability(Ability::Breathe));
        if !(ready && has_target && target_info.line_of_sight && wanted) {
            continue;
        }

        // The body can sit exactly on the target, where the bearing collapses
        // to zero. There is no angle to fire along, so wait for separation
        // instead of launching a NaN projectile.
        let aim_direction = target_info.direction;
        if aim_direction.length_squared() <= f32::EPSILON {
            continue;
        }

        for template in templates.iter() {
            commands.trigger(FireProjectile {
                item: template,
                side: *side,
                holder: Some(entity),
                position: transform.translation.xy(),
                aim_direction,
            });
        }
        commands.trigger(ActionFinished {
            entity,
            action: Some(Action::Ability(Ability::Breathe)),
        });
    }
}
