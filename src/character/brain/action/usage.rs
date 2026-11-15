use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{Action, ActionFinished, ActionOutcome};
use crate::character::ability::{
    Ability, AbilityRead, AbilityUsage, AbilityUseFailure, AbilityUseFinished,
};
use crate::character::brain::{Brain, CapabilityProfile};
use crate::prelude::*;

/// Whether an action could start now; see [`ActionRead::readiness`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::character::brain) enum ActionReadiness {
    Ready,
    /// Only waiting stands in the way, such as a cooldown, mana or an interruption.
    Waiting,
    /// The character lacks what the action needs, such as the item, the ability or a potion.
    Missing,
}

impl ActionReadiness {
    fn from_result<T, E: Copy>(result: Result<T, E>, is_temporary: fn(E) -> bool) -> Self {
        match result {
            Ok(_) => Self::Ready,
            Err(failure) if is_temporary(failure) => Self::Waiting,
            Err(_) => Self::Missing,
        }
    }
}

/// Reads the item and ability uses behind the brain's actions: what each reaches, whether it can
/// start, and which are in progress.
#[derive(SystemParam)]
pub(in crate::character::brain) struct ActionRead<'w, 's> {
    equipment: EquipmentRead<'w, 's>,
    abilities: AbilityRead<'w, 's>,
}

impl ActionRead<'_, '_> {
    /// The profile of the capability that carries out `holder`'s `action`: the equipped item's
    /// reach, or the ability's. `Idle` needs none.
    pub fn profile(&self, holder: Entity, action: Action) -> Option<CapabilityProfile> {
        let reach = match action {
            Action::Use(slot) => self.equipment.equippable_in_slot(holder, slot)?.reach,
            Action::Ability(ability) => {
                if !self.abilities.has(holder, ability) {
                    return None;
                }
                self.abilities.reach(holder, ability)
            }
            Action::Idle => None,
        };
        Some(CapabilityProfile {
            reach,
            ..CapabilityProfile::default()
        })
    }

    /// Whether `holder` could start `action` now, or what stands in the way.
    pub fn readiness(&self, holder: Entity, action: Action) -> ActionReadiness {
        match action {
            Action::Use(slot) => ActionReadiness::from_result(
                self.equipment.can_use(holder, slot),
                EquipmentUseFailure::is_temporary,
            ),
            Action::Ability(ability) => ActionReadiness::from_result(
                self.abilities.can_use(holder, ability),
                AbilityUseFailure::is_temporary,
            ),
            Action::Idle => ActionReadiness::Ready,
        }
    }

    /// Whether `holder`'s `action` ends on its own, so a committed behavior over it can finish:
    /// an instant item use or an ability. A held use ends only when released.
    pub fn ends_on_its_own(&self, holder: Entity, action: Action) -> bool {
        match action {
            Action::Use(slot) => self
                .equipment
                .equippable_in_slot(holder, slot)
                .is_some_and(|equippable| equippable.use_style.ends_on_its_own()),
            Action::Ability(_) => true,
            Action::Idle => false,
        }
    }

    /// The actions `holder`'s items and abilities are carrying out now, whether or not the brain
    /// still holds them.
    pub fn actions_in_progress(&self, holder: Entity) -> impl Iterator<Item = Action> + '_ {
        let equipment_uses = EquipmentSlot::ALL
            .into_iter()
            .filter(move |&slot| self.equipment.is_using(holder, slot))
            .map(Action::Use);
        let ability_uses = self.abilities.abilities_in_use(holder).map(Action::Ability);
        equipment_uses.chain(ability_uses)
    }
}

/// Starts and releases the item and ability uses behind the brain's actions.
#[derive(SystemParam)]
pub(super) struct ActionUsage<'w, 's> {
    equipment: EquipmentUsage<'w, 's>,
    abilities: AbilityUsage<'w, 's>,
}

impl ActionUsage<'_, '_> {
    fn is_in_progress(&self, holder: Entity, action: Action) -> bool {
        match action {
            Action::Use(slot) => self.equipment.is_using(holder, slot),
            Action::Ability(ability) => self.abilities.is_using(holder, ability),
            Action::Idle => false,
        }
    }

    fn try_start(&mut self, holder: Entity, action: Action) -> bool {
        match action {
            Action::Use(slot) => self.equipment.try_use(holder, slot).is_ok(),
            Action::Ability(ability) => self.abilities.try_use(holder, ability).is_ok(),
            Action::Idle => false,
        }
    }

    fn release_held_use(&mut self, holder: Entity, action: Action) {
        match action {
            Action::Use(slot) => self.equipment.release_held_use(holder, slot),
            Action::Ability(_) | Action::Idle => {}
        }
    }
}

/// The behavior run id of the last use started of each action, so a committed action starts once
/// and its finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in crate::character::brain) struct StartedActions {
    uses: [Option<u32>; EquipmentSlot::ALL.len()],
    abilities: [Option<u32>; Ability::ALL.len()],
}

impl StartedActions {
    fn behavior_run_id_mut(&mut self, action: Action) -> Option<&mut Option<u32>> {
        match action {
            Action::Use(slot) => Some(&mut self.uses[slot as usize]),
            Action::Ability(ability) => Some(&mut self.abilities[ability as usize]),
            Action::Idle => None,
        }
    }

    fn behavior_run_id(&self, action: Action) -> Option<u32> {
        match action {
            Action::Use(slot) => self.uses[slot as usize],
            Action::Ability(ability) => self.abilities[ability as usize],
            Action::Idle => None,
        }
    }
}

/// Starts the brain's held actions and releases held uses it no longer holds. Considerations
/// already judged the target, so the game decides only whether a use can start. A committed action
/// starts once, or adopts a use of it already in progress so that use's finish ends the behavior;
/// a continuously scored one starts again whenever it isn't in progress.
pub(super) fn apply_action_requests(
    mut commands: Commands,
    mut usage: ActionUsage,
    mut brains: Query<(Entity, &Brain, &mut StartedActions)>,
) {
    for (entity, brain, mut started_actions) in &mut brains {
        for slot in EquipmentSlot::ALL {
            let action = Action::Use(slot);
            if brain.held_action(action).is_none() {
                usage.release_held_use(entity, action);
            }
        }
        for &running in &brain.running_behaviors {
            let Some(action) = running.action else {
                continue;
            };
            let Some(started_behavior_run_id) = started_actions.behavior_run_id_mut(action) else {
                continue;
            };
            if running.committed && *started_behavior_run_id == Some(running.behavior_run_id) {
                continue;
            }
            if usage.is_in_progress(entity, action) {
                if running.committed {
                    *started_behavior_run_id = Some(running.behavior_run_id);
                }
                continue;
            }
            if usage.try_start(entity, action) {
                *started_behavior_run_id = Some(running.behavior_run_id);
            } else {
                running.report_not_started(&mut commands, entity);
            }
        }
    }
}

/// Reports [`ActionFinished`] for the use whose item finished.
pub(super) fn on_report_equipment_use_finished(
    finished: On<EquipmentUseFinished>,
    mut commands: Commands,
    started_actions: Query<&StartedActions>,
) {
    report_action_finished(
        &mut commands,
        &started_actions,
        finished.holder,
        Action::Use(finished.slot),
    );
}

/// Reports [`ActionFinished`] for the use whose ability finished.
pub(super) fn on_report_ability_use_finished(
    finished: On<AbilityUseFinished>,
    mut commands: Commands,
    started_actions: Query<&StartedActions>,
) {
    report_action_finished(
        &mut commands,
        &started_actions,
        finished.holder,
        Action::Ability(finished.ability),
    );
}

fn report_action_finished(
    commands: &mut Commands,
    started_actions: &Query<&StartedActions>,
    holder: Entity,
    action: Action,
) {
    let Some(behavior_run_id) = started_actions
        .get(holder)
        .ok()
        .and_then(|started_actions| started_actions.behavior_run_id(action))
    else {
        return;
    };
    commands.trigger(ActionFinished {
        entity: holder,
        action,
        behavior_run_id,
        outcome: ActionOutcome::Completed,
    });
}
