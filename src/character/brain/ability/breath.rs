use bevy::prelude::*;

use super::{AbilityCapability, Cooldown};
use crate::{
    character::{
        brain::{Ability, Action, ActionFinished, ActionIntent, CAST_MAX_RANGE, Tuning},
        vision::{TargetInfo, Targeting},
    },
    prelude::*,
};

/// An innate projectile attack that needs no weapon. The projectile template
/// rides on the entity as a `Projectiles` relation. Opt-in at spawn; absent
/// means the entity never breathes.
#[derive(Component, Debug)]
pub struct Breath {
    pub cooldown: Cooldown,
    /// Farthest target distance at which the brain breathes.
    pub reach: f32,
}

impl Breath {
    /// Breathes at most once per `period` seconds within cast range; the
    /// first breath waits one period.
    pub fn new(period: f32) -> Self {
        Self {
            cooldown: Cooldown::delayed(period),
            reach: CAST_MAX_RANGE,
        }
    }
}

impl AbilityCapability for Breath {
    fn ability(&self) -> Ability {
        Ability::Breathe
    }

    fn tuning(&self) -> Tuning {
        Tuning {
            reach: Some(self.reach),
            ..Tuning::default()
        }
    }
}

/// Launches the innate projectile whenever the brain wants to breathe at a
/// sighted target.
pub(super) fn apply_breath(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut breathers: Query<(
        Entity,
        &ActionIntent,
        Has<Targeting>,
        &TargetInfo,
        &Transform,
        &CombatSide,
        &Projectiles,
        &mut Breath,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, intent, has_target, target_info, transform, side, templates, mut breath) in
        &mut breathers
    {
        // Checked first so the opening delay counts from spawn, not from the
        // first wanted breath.
        let ready = breath.cooldown.is_ready(now);
        let wanted = intent.get(Action::Ability(Ability::Breathe));
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
        breath.cooldown.start(now);
        commands.trigger(ActionFinished { entity });
    }
}
