use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{Action, ActionFinished, ActionOutcome};
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

/// Reads the uses behind the brain's actions: what each reaches, whether it can start, and which
/// are in progress.
#[derive(SystemParam)]
pub(in crate::character::brain) struct ActionRead<'w, 's> {
    uses: UseRead<'w, 's>,
}

impl ActionRead<'_, '_> {
    /// The profile of the usable that carries out `holder`'s `action`: its reach. `Idle` needs
    /// none.
    pub fn profile(&self, holder: Entity, action: Action) -> Option<CapabilityProfile> {
        match action {
            Action::Use(slot) => {
                self.uses.usable_in_slot(holder, slot)?;
                Some(CapabilityProfile {
                    reach: self.uses.reach(holder, slot),
                    ..CapabilityProfile::default()
                })
            }
            Action::Idle => None,
        }
    }

    /// Whether `holder` could start `action` now, or what stands in the way.
    pub fn readiness(&self, holder: Entity, action: Action) -> ActionReadiness {
        match action {
            Action::Use(slot) => match self.uses.can_use(holder, slot) {
                Ok(_) => ActionReadiness::Ready,
                Err(failure) if failure.is_temporary() => ActionReadiness::Waiting,
                Err(_) => ActionReadiness::Missing,
            },
            Action::Idle => ActionReadiness::Ready,
        }
    }

    /// Whether `holder`'s `action` ends on its own, so a committed behavior over it can finish;
    /// see [`UseStyle`].
    pub fn ends_on_its_own(&self, holder: Entity, action: Action) -> bool {
        match action {
            Action::Use(slot) => self.uses.ends_on_its_own(holder, slot),
            Action::Idle => false,
        }
    }

    /// The actions `holder`'s usables are carrying out now, whether or not a running behavior
    /// still chooses them.
    pub fn actions_in_progress(&self, holder: Entity) -> impl Iterator<Item = Action> + '_ {
        self.uses.slots_in_use(holder).map(Action::Use)
    }
}

/// The behavior run id of the last use started in each slot, so a committed action starts once
/// and its finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub(in crate::character::brain) struct StartedActions([Option<u32>; UseSlot::COUNT]);

/// Starts the actions of the brain's running behaviors and stops sustained uses none of them
/// still chooses. Considerations already judged the target, so the game decides only whether a use
/// can start. A committed action starts once, or adopts a use of it already in progress so that
/// use's finish ends the behavior; a continuously scored one starts again whenever it isn't in
/// progress.
pub(super) fn apply_action_requests(
    mut commands: Commands,
    mut uses: UseCommands,
    mut brains: Query<(Entity, &Brain, &mut StartedActions)>,
) {
    for (entity, brain, mut started_actions) in &mut brains {
        for slot in UseSlot::ALL {
            if brain.running_behavior_with(Action::Use(slot)).is_none() {
                uses.stop_sustained_use(entity, slot);
            }
        }
        for &running in &brain.running_behaviors {
            let slot = match running.action {
                Some(Action::Use(slot)) => slot,
                Some(Action::Idle) | None => continue,
            };
            let started_behavior_run_id = &mut started_actions[slot.index()];
            if running.committed && *started_behavior_run_id == Some(running.behavior_run_id) {
                continue;
            }
            if uses.read().is_using(entity, slot) {
                if running.committed {
                    *started_behavior_run_id = Some(running.behavior_run_id);
                }
                continue;
            }
            if uses.try_use(entity, slot).is_ok() {
                *started_behavior_run_id = Some(running.behavior_run_id);
            } else {
                running.report_not_started(&mut commands, entity);
            }
        }
    }
}

/// Reports [`ActionFinished`] for the use that finished.
pub(super) fn on_use_finished(
    finished: On<UseFinished>,
    mut commands: Commands,
    started_actions: Query<&StartedActions>,
) {
    let Some(behavior_run_id) = started_actions
        .get(finished.holder)
        .ok()
        .and_then(|started_actions| started_actions[finished.slot.index()])
    else {
        return;
    };
    commands.trigger(ActionFinished {
        entity: finished.holder,
        action: Action::Use(finished.slot),
        behavior_run_id,
        outcome: ActionOutcome::Completed,
    });
}
