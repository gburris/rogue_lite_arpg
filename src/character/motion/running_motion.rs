use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    DashProgress, Motion, MotionReach, MotionSources, OrbitProgress, Steering, SwoopProgress,
    TunedBy, WanderProgress,
};
use crate::prelude::*;

/// Links a running motion to the character it moves; see [`RunningMotion`].
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = CurrentMotion)]
pub struct MotionOf {
    pub character: Entity,
}

/// The running motion of this character, if it runs one; see [`RunningMotion`].
#[derive(Component, Debug)]
#[relationship_target(relationship = MotionOf, linked_spawn)]
pub struct CurrentMotion(Entity);

impl CurrentMotion {
    pub fn get(&self) -> Entity {
        self.0
    }
}

/// A character's running motion, on its own entity linked to the character by [`MotionOf`] and to
/// the motion source that tunes it by [`TunedBy`]: the motion and the entity it steers by.
/// [`MotionCommands`] spawns it with the motion's progress and [`Steering`], and a
/// [`FiniteMotion`] despawns it when it ends. Despawning it, however the motion ended, halts the
/// character and, for a finite motion, starts the source's [`Cooldown`] and fires
/// [`MotionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[require(Steering)]
#[component(on_remove = end_running_motion)]
pub(in crate::character) struct RunningMotion {
    pub motion: Motion,
    pub target: Option<Entity>,
}

/// Marks a running motion that ends on its own; see [`Motion::is_finite`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct FiniteMotion;

fn end_running_motion(mut world: DeferredWorld, context: HookContext) {
    let Ok((&MotionOf { character }, &running, tuned_by, is_finite)) =
        world.entity(context.entity).get_components::<(
            &MotionOf,
            &RunningMotion,
            Option<&TunedBy>,
            Has<FiniteMotion>,
        )>()
    else {
        return;
    };
    let tuned_by = tuned_by.copied();

    if let Some(mut kinematics) = world.get_mut::<Kinematics>(character) {
        kinematics.stop_moving();
    }

    if !is_finite {
        return;
    }
    let now = world.resource::<Time<Virtual>>().elapsed_secs();
    if let Some(mut cooldown) =
        tuned_by.and_then(|tuned_by| world.get_mut::<Cooldown>(tuned_by.source))
    {
        cooldown.start(now);
    }
    world.commands().trigger(MotionFinished {
        entity: character,
        motion: running.motion,
    });
}

/// A finite motion ended: it completed, lost its target, was interrupted (see
/// [`InterruptionState`]), was replaced or was stopped.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
}

/// What [`MotionStatus`] reads of a character: its running motion and motion sources.
#[derive(QueryData)]
struct CharacterMotionData {
    current_motion: Option<&'static CurrentMotion>,
    sources: Option<&'static MotionSources>,
}

/// What [`MotionStatus`] reads of one motion source.
#[derive(QueryData)]
struct MotionSourceData {
    entity: Entity,
    motion: &'static Motion,
    cooldown: Option<&'static Cooldown>,
    reach: Option<&'static MotionReach>,
}

/// Reads which motion a character runs and which motions it could start.
#[derive(SystemParam)]
pub(in crate::character) struct MotionStatus<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    characters: Query<'w, 's, CharacterMotionData>,
    interruptions: Query<'w, 's, InterruptionState>,
    running_motions: Query<'w, 's, &'static RunningMotion>,
    sources: Query<'w, 's, MotionSourceData>,
}

impl MotionStatus<'_, '_> {
    /// The motion `character` runs and its target; `None` while it runs none.
    pub(in crate::character) fn running_motion(&self, character: Entity) -> Option<RunningMotion> {
        self.current_motion(character).map(|(_, running)| running)
    }

    /// Whether a finite `motion` is in progress on `character`. Always false for a
    /// continuous motion.
    pub(in crate::character) fn is_in_progress(&self, character: Entity, motion: Motion) -> bool {
        motion.is_finite()
            && self
                .running_motion(character)
                .is_some_and(|running| running.motion == motion)
    }

    /// Whether `character` can start `motion` now: it has a source for it if it
    /// needs one (see [`Motion::needs_source`]), the source's [`Cooldown`] is ready, and
    /// for a finite motion the character is not interrupted (see [`InterruptionState`]).
    pub(in crate::character) fn is_ready(&self, character: Entity, motion: Motion) -> bool {
        let is_interrupted = self
            .interruptions
            .get(character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if motion.is_finite() && is_interrupted {
            return false;
        }
        let Some(source) = self.source_data(character, motion) else {
            return !motion.needs_source();
        };
        let now = self.time.elapsed_secs();
        source
            .cooldown
            .is_none_or(|cooldown| cooldown.is_ready(now))
    }

    /// The farthest target distance at which `motion` counts as in reach for
    /// `character`; `None` without a source or a [`MotionReach`].
    pub(in crate::character) fn reach(&self, character: Entity, motion: Motion) -> Option<f32> {
        let source = self.source_data(character, motion)?;
        source.reach.map(|&MotionReach(reach)| reach)
    }

    fn current_motion(&self, character: Entity) -> Option<(Entity, RunningMotion)> {
        let character_data = self.characters.get(character).ok()?;
        let entity = character_data.current_motion?.get();
        let running = self.running_motions.get(entity).ok()?;
        Some((entity, *running))
    }

    fn source_for_motion(&self, character: Entity, motion: Motion) -> Option<Entity> {
        self.source_data(character, motion)
            .map(|source| source.entity)
    }

    fn source_data(
        &self,
        character: Entity,
        motion: Motion,
    ) -> Option<MotionSourceDataItem<'_, '_>> {
        let character_data = self.characters.get(character).ok()?;
        character_data.sources?.iter().find_map(|source| {
            self.sources
                .get(source)
                .ok()
                .filter(|source_data| *source_data.motion == motion)
        })
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

    /// Runs `motion` on `character`, steering by `target`. A continuous motion that is already
    /// running keeps its progress and takes the new target; anything else replaces the running
    /// motion with a fresh one.
    pub(in crate::character) fn start(
        &mut self,
        character: Entity,
        motion: Motion,
        target: Option<Entity>,
    ) {
        let running = RunningMotion { motion, target };
        let current_motion = self.status.current_motion(character);
        if let Some((entity, current)) = current_motion
            && !motion.is_finite()
            && current.motion == motion
        {
            self.commands.entity(entity).try_insert(running);
            return;
        }

        if let Some((entity, _)) = current_motion {
            self.commands.entity(entity).try_despawn();
        }
        let mut running_motion =
            self.commands
                .spawn((Name::new("Running motion"), MotionOf { character }, running));
        if let Some(source) = self.status.source_for_motion(character, motion) {
            running_motion.insert(TunedBy { source });
        }
        if motion.is_finite() {
            running_motion.insert(FiniteMotion);
        }
        match motion {
            Motion::Dash => {
                running_motion.insert(DashProgress::default());
            }
            Motion::Swoop => {
                running_motion.insert(SwoopProgress::default());
            }
            Motion::Orbit => {
                running_motion.insert(OrbitProgress::default());
            }
            Motion::Wander => {
                running_motion.insert(WanderProgress::default());
            }
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance
            | Motion::Idle => {}
        }
    }

    /// Changes the target of `character`'s running motion, keeping its progress.
    pub(in crate::character) fn retarget(&mut self, character: Entity, target: Option<Entity>) {
        if let Some((entity, running)) = self.status.current_motion(character) {
            self.commands
                .entity(entity)
                .try_insert(RunningMotion { target, ..running });
        }
    }

    /// Ends `character`'s running motion.
    pub(in crate::character) fn stop(&mut self, character: Entity) {
        if let Some((entity, _)) = self.status.current_motion(character) {
            self.commands.entity(entity).try_despawn();
        }
    }
}

/// Ends each finite motion whose character is interrupted; see [`InterruptionState`].
pub(super) fn end_interrupted_motions(
    mut commands: Commands,
    running_motions: Query<(Entity, &MotionOf), With<FiniteMotion>>,
    characters: Query<InterruptionState>,
) {
    for (entity, motion_of) in &running_motions {
        let is_interrupted = characters
            .get(motion_of.character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if is_interrupted {
            commands.entity(entity).try_despawn();
        }
    }
}
