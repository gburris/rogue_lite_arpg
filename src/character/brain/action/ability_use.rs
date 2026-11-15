use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use super::{Action, ActionFinished, ActionOutcome};
use crate::character::ability::{Ability, AbilityUsage, AbilityUseFinished};
use crate::character::brain::{Brain, Motion, TargetInfo, Targeting};

/// How far past its reach a passing biter still snaps, as a multiple of the reach.
const PASSING_BITE_REACH_FACTOR: f32 = 2.0;

/// The behavior run id of the last use started of each ability, so a committed use starts once
/// and its ability's finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in crate::character::brain) struct StartedAbilityUses([Option<u32>; Ability::ALL.len()]);

/// Reports [`ActionFinished`] with the run id of the use whose ability finished.
pub(super) fn on_report_ability_use_finished(
    finished: On<AbilityUseFinished>,
    mut commands: Commands,
    started_ability_uses: Query<&StartedAbilityUses>,
) {
    let Ok(started_ability_uses) = started_ability_uses.get(finished.holder) else {
        return;
    };
    if let Some(behavior_run_id) = started_ability_uses.0[finished.ability as usize] {
        commands.trigger(ActionFinished {
            entity: finished.holder,
            action: Action::Ability(finished.ability),
            behavior_run_id,
            outcome: ActionOutcome::Completed,
        });
    }
}

/// Uses the brain's held ability once the live target suits it. Decisions sample at 10 Hz, so the
/// target is re-checked every frame: a breath needs a sighted target and no dash or swoop under
/// way, and a bite a sighted target within reach, or a pass inside [`PASSING_BITE_REACH_FACTOR`]
/// times the reach that starts to pull away, its closest point. A committed use starts once; a
/// continuously scored one again whenever its ability allows.
pub(super) fn apply_ability_requests(
    mut commands: Commands,
    mut ability_usage: AbilityUsage,
    mut requests: Query<(
        Entity,
        &Brain,
        &mut StartedAbilityUses,
        Has<Targeting>,
        Option<&TargetInfo>,
        Option<&LinearVelocity>,
    )>,
) {
    for (entity, brain, mut started_ability_uses, has_target, target_info, linear_velocity) in
        &mut requests
    {
        for ability in Ability::ALL {
            let Some(requested) = brain.held_action(Action::Ability(ability)) else {
                continue;
            };
            let started_use = &mut started_ability_uses.0[ability as usize];
            if requested.committed && *started_use == Some(requested.behavior_run_id) {
                continue;
            }
            let own_velocity = linear_velocity.map_or(Vec2::ZERO, |velocity| velocity.0);
            let reach = ability_usage.reach(entity, ability);
            let suits =
                suits_live_target(ability, brain, has_target, target_info, own_velocity, reach);
            if !suits {
                requested.report_not_started(&mut commands, entity);
                continue;
            }
            match ability_usage.try_use(entity, ability) {
                Ok(()) => *started_use = Some(requested.behavior_run_id),
                Err(_) => requested.report_not_started(&mut commands, entity),
            }
        }
    }
}

fn suits_live_target(
    ability: Ability,
    brain: &Brain,
    has_target: bool,
    target_info: Option<&TargetInfo>,
    own_velocity: Vec2,
    reach: Option<f32>,
) -> bool {
    match ability {
        Ability::Heal | Ability::Rally => true,
        Ability::Breathe => {
            // The decision that starts a dash or swoop can still hold a breath.
            let bursting = brain.motion().is_some_and(Motion::is_finite);
            let sighted = target_info.is_some_and(|target_info| {
                target_info.line_of_sight && target_info.nonzero_direction().is_some()
            });
            has_target && sighted && !bursting
        }
        Ability::Bite => {
            let (Some(target_info), Some(reach)) = (target_info, reach) else {
                return false;
            };
            let closing_velocity = target_info.velocity - own_velocity;
            let pulling_away = closing_velocity.dot(target_info.direction) > 0.0;
            let passing = pulling_away && target_info.is_within(reach * PASSING_BITE_REACH_FACTOR);
            has_target && target_info.line_of_sight && (target_info.is_within(reach) || passing)
        }
    }
}
