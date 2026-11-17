use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{
    Dash, DashProgress, Motion, Orbit, OrbitProgress, Swoop, SwoopProgress, Wander, WanderProgress,
};
use crate::prelude::*;

/// The motion a character runs and the entity it steers by, set through [`MotionCommands`]. A
/// finite motion also has a progress component; removing it ends the motion and fires
/// [`MotionFinished`]. This component stays until the next request.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::character) struct RunningMotion {
    pub motion: Motion,
    pub target: Option<Entity>,
}

/// A finite motion ended: it completed, lost its target, was interrupted (see
/// [`InterruptionState`]), or was replaced.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
}

/// What [`MotionStatus`] reads of a character: its running motion and motion capabilities.
#[derive(QueryData)]
struct CharacterMotionData {
    running: Option<&'static RunningMotion>,
    dash: Option<&'static Dash>,
    swoop: Option<&'static Swoop>,
    has_orbit: Has<Orbit>,
    has_wander: Has<Wander>,
    has_dash_progress: Has<DashProgress>,
    has_swoop_progress: Has<SwoopProgress>,
    interruption: InterruptionState,
}

/// Reads which motion a character runs and which motions it could start.
#[derive(SystemParam)]
pub(in crate::character) struct MotionStatus<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    characters: Query<'w, 's, CharacterMotionData>,
}

impl MotionStatus<'_, '_> {
    /// The motion `character` runs and its target; `None` while it runs none.
    pub(in crate::character) fn running_motion(&self, character: Entity) -> Option<RunningMotion> {
        let character_data = self.characters.get(character).ok()?;
        character_data.running.copied()
    }

    /// Whether the finite `motion` is in progress on `character`. Always false for a continuous
    /// motion.
    pub(in crate::character) fn is_in_progress(&self, character: Entity, motion: Motion) -> bool {
        let Ok(character_data) = self.characters.get(character) else {
            return false;
        };
        match motion {
            Motion::Dash => character_data.has_dash_progress,
            Motion::Swoop => character_data.has_swoop_progress,
            Motion::Wander
            | Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle
            | Motion::Orbit(_) => false,
        }
    }

    /// Whether `character` can start `motion` now: it has the capability and, for a finite motion,
    /// the cooldown is ready and the character is not interrupted (see [`InterruptionState`]).
    pub(in crate::character) fn is_ready(&self, character: Entity, motion: Motion) -> bool {
        let Ok(character_data) = self.characters.get(character) else {
            return false;
        };

        let now = self.time.elapsed_secs();
        let is_interrupted = character_data.interruption.is_interrupted();
        match motion {
            Motion::Dash => {
                !is_interrupted
                    && character_data
                        .dash
                        .is_some_and(|dash| dash.cooldown().is_ready(now))
            }
            Motion::Swoop => {
                !is_interrupted
                    && character_data
                        .swoop
                        .is_some_and(|swoop| swoop.cooldown().is_ready(now))
            }
            Motion::Orbit(_) => character_data.has_orbit,
            Motion::Wander => character_data.has_wander,
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle => true,
        }
    }

    /// The farthest target distance at which `motion` counts as in reach for `character`; `None`
    /// without the capability or a reach.
    pub(in crate::character) fn reach(&self, character: Entity, motion: Motion) -> Option<f32> {
        let character_data = self.characters.get(character).ok()?;
        match motion {
            Motion::Dash => character_data.dash.and_then(Dash::reach),
            Motion::Swoop => character_data.swoop.map(Swoop::reach),
            Motion::Wander
            | Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle
            | Motion::Orbit(_) => None,
        }
    }
}

/// Starts and stops motions.
#[derive(SystemParam)]
pub(in crate::character) struct MotionCommands<'w, 's> {
    commands: Commands<'w, 's>,
    status: MotionStatus<'w, 's>,
}

impl<'w, 's> MotionCommands<'w, 's> {
    /// The status these commands check against.
    pub(in crate::character) fn status(&self) -> &MotionStatus<'w, 's> {
        &self.status
    }

    /// Sets `motion` as `character`'s running motion, steering by `target`, and removes the
    /// progress of any other motion. A finite motion starts with fresh progress. A continuous
    /// motion of the kind already running keeps its progress.
    pub(in crate::character) fn start(
        &mut self,
        character: Entity,
        motion: Motion,
        target: Option<Entity>,
    ) {
        let keeps_progress = !motion.is_finite()
            && self
                .status
                .running_motion(character)
                .is_some_and(|running| running.motion.is_same_kind(motion));

        let mut entity = self.commands.entity(character);
        entity.try_insert(RunningMotion { motion, target });
        if keeps_progress {
            return;
        }

        // Inserting over existing progress replaces it without running its removal hook.
        match motion {
            Motion::Dash => {
                entity
                    .try_remove::<(SwoopProgress, OrbitProgress, WanderProgress)>()
                    .try_insert(DashProgress::default());
            }
            Motion::Swoop => {
                entity
                    .try_remove::<(DashProgress, OrbitProgress, WanderProgress)>()
                    .try_insert(SwoopProgress::default());
            }
            Motion::Orbit(_) => {
                entity
                    .try_remove::<(DashProgress, SwoopProgress, WanderProgress)>()
                    .try_insert(OrbitProgress::default());
            }
            Motion::Wander => {
                entity
                    .try_remove::<(DashProgress, SwoopProgress, OrbitProgress)>()
                    .try_insert(WanderProgress::default());
            }
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle => {
                entity.try_remove::<(DashProgress, SwoopProgress, OrbitProgress, WanderProgress)>();
            }
        }
    }

    /// Changes the target of `character`'s running motion, keeping its progress.
    pub(in crate::character) fn retarget(&mut self, character: Entity, target: Option<Entity>) {
        if let Some(running) = self.status.running_motion(character) {
            self.commands.entity(character).try_insert(RunningMotion {
                motion: running.motion,
                target,
            });
        }
    }

    /// Removes `character`'s running motion and its progress.
    pub(in crate::character) fn stop(&mut self, character: Entity) {
        self.commands.entity(character).try_remove::<(
            RunningMotion,
            DashProgress,
            SwoopProgress,
            OrbitProgress,
            WanderProgress,
        )>();
    }
}

/// Ends the finite motion of each interrupted character; see [`InterruptionState`].
pub(super) fn end_interrupted_motions(
    mut commands: Commands,
    characters: Query<(Entity, InterruptionState), Or<(With<DashProgress>, With<SwoopProgress>)>>,
) {
    for (entity, interruption) in &characters {
        if interruption.is_interrupted() {
            commands
                .entity(entity)
                .try_remove::<(DashProgress, SwoopProgress)>();
        }
    }
}
