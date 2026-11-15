use bevy::prelude::*;

use super::{Action, ActionFinished, ActionOutcome};
use crate::character::ability::{Ability, AbilityUsage, AbilityUseFinished};
use crate::character::brain::Brain;

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

/// Uses the brain's held ability. Considerations already judged the target, so the ability
/// itself decides only whether it can start; see [`AbilityUsage::try_use`]. A committed use starts
/// once; a continuously scored one again whenever its ability allows.
pub(super) fn apply_ability_requests(
    mut commands: Commands,
    mut ability_usage: AbilityUsage,
    mut requests: Query<(Entity, &Brain, &mut StartedAbilityUses)>,
) {
    for (entity, brain, mut started_ability_uses) in &mut requests {
        for ability in Ability::ALL {
            let Some(requested) = brain.held_action(Action::Ability(ability)) else {
                continue;
            };
            let started_use = &mut started_ability_uses.0[ability as usize];
            if requested.committed && *started_use == Some(requested.behavior_run_id) {
                continue;
            }
            match ability_usage.try_use(entity, ability) {
                Ok(()) => *started_use = Some(requested.behavior_run_id),
                Err(_) => requested.report_not_started(&mut commands, entity),
            }
        }
    }
}
