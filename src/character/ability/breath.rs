use bevy::prelude::*;

use super::AbilityCapability;
use crate::{
    character::{
        brain::{Ability, AbilityIntent, AbilityTuning},
        vision::{TargetInfo, Targeting},
    },
    prelude::*,
};

/// An innate projectile attack that needs no weapon. The projectile template
/// rides on the entity as a `Projectiles` relation. Opt-in at spawn; absent
/// means the entity never breathes.
#[derive(Component, Debug)]
pub struct Breath {
    cooldown: Timer,
}

impl Breath {
    /// Breathes at most once per `period` seconds; the first breath waits one period.
    pub fn new(period: f32) -> Self {
        Self {
            cooldown: Timer::from_seconds(period, TimerMode::Once),
        }
    }
}

impl AbilityCapability for Breath {
    fn ability(&self) -> Ability {
        Ability::Breathe
    }

    fn tuning(&self) -> AbilityTuning {
        AbilityTuning::default()
    }
}

/// Launches the innate projectile whenever the brain wants to breathe at a
/// sighted target.
pub(super) fn apply_breath(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut breathers: Query<(
        Entity,
        &AbilityIntent,
        Has<Targeting>,
        &TargetInfo,
        &Health,
        &Transform,
        &CombatSide,
        &Projectiles,
        &mut Breath,
    )>,
) {
    for (entity, intent, has_target, target_info, health, transform, side, templates, mut breath) in
        &mut breathers
    {
        // The defeated body lingers for its dying beat; the breath stops at 0 hp.
        if health.hp <= 0.0 {
            continue;
        }
        breath.cooldown.tick(time.delta());
        if !(has_target && target_info.line_of_sight && intent.get(Ability::Breathe)) {
            continue;
        }
        if !breath.cooldown.is_finished() {
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
        breath.cooldown.reset();
    }
}
