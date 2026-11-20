use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{FiniteMotion, MotionKey, MotionReach, MotionSources, Steering, TunedBy};
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
/// the motion source that tunes it by [`TunedBy`]: the source's key and the entity it steers by.
/// [`MotionCommands`] spawns it with [`Steering`], [`on_motion_started`] adds that motion's
/// [`MotionProgress`], and a finite motion's system despawns it when it ends. Despawning it,
/// however the motion ended, halts the character and, for a [`FiniteMotion`], starts the source's
/// [`Cooldown`] and fires [`MotionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[require(Steering)]
#[component(on_remove = end_running_motion)]
pub(in crate::character) struct RunningMotion {
    pub key: MotionKey,
    pub target: Option<Entity>,
}

/// A motion's state while it runs, which [`on_motion_started`] adds to its running motion.
pub(super) trait MotionProgress: Component + Default {
    /// The tuning component of the motion sources whose running motions get this progress.
    type Tuning: Component;
}

/// Gives each newly started running motion tuned by a `Progress::Tuning` source a fresh
/// `Progress`.
pub(super) fn on_motion_started<Progress: MotionProgress>(
    added: On<Add<RunningMotion>>,
    mut commands: Commands,
    running_motions: Query<&TunedBy>,
    tunings: Query<(), With<Progress::Tuning>>,
) {
    let is_progress_motion = running_motions
        .get(added.entity)
        .is_ok_and(|tuned_by| tunings.contains(tuned_by.source));
    if is_progress_motion {
        commands.entity(added.entity).insert(Progress::default());
    }
}

fn end_running_motion(mut world: DeferredWorld, context: HookContext) {
    let Ok((&MotionOf { character }, &running, tuned_by)) = world
        .entity(context.entity)
        .get_components::<(&MotionOf, &RunningMotion, Option<&TunedBy>)>()
    else {
        return;
    };
    let tuned_by = tuned_by.copied();

    if let Some(mut kinematics) = world.get_mut::<Kinematics>(character) {
        kinematics.stop_moving();
    }

    let Some(TunedBy { source }) = tuned_by else {
        return;
    };
    let is_finite = world
        .get_entity(source)
        .is_ok_and(|source| source.contains::<FiniteMotion>());
    if !is_finite {
        return;
    }
    let now = world.resource::<Time<Virtual>>().elapsed_secs();
    if let Some(mut cooldown) = world.get_mut::<Cooldown>(source) {
        cooldown.start(now);
    }
    world.commands().trigger(MotionFinished {
        entity: character,
        key: running.key,
    });
}

/// A finite motion ended: it completed, lost its target, was interrupted (see
/// [`InterruptionState`]), was replaced or was stopped.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFinished {
    pub entity: Entity,
    pub key: MotionKey,
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
    key: &'static MotionKey,
    is_finite: Has<FiniteMotion>,
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

    /// Whether `character` runs the finite motion of its `key` source. Always false for a
    /// continuous motion.
    pub(in crate::character) fn is_in_progress(&self, character: Entity, key: MotionKey) -> bool {
        self.ends_on_its_own(character, key)
            && self
                .running_motion(character)
                .is_some_and(|running| running.key == key)
    }

    /// Whether `character` has a `key` source whose motion ends on its own; see
    /// [`FiniteMotion`].
    pub(in crate::character) fn ends_on_its_own(&self, character: Entity, key: MotionKey) -> bool {
        self.source_data(character, key)
            .is_some_and(|source| source.is_finite)
    }

    /// Whether `character` can start the motion of its `key` source now: it has the source, the
    /// source's [`Cooldown`] is ready, and for a finite motion the character is not interrupted
    /// (see [`InterruptionState`]).
    pub(in crate::character) fn is_ready(&self, character: Entity, key: MotionKey) -> bool {
        let Some(source) = self.source_data(character, key) else {
            return false;
        };
        let is_interrupted = self
            .interruptions
            .get(character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if source.is_finite && is_interrupted {
            return false;
        }
        let now = self.time.elapsed_secs();
        source
            .cooldown
            .is_none_or(|cooldown| cooldown.is_ready(now))
    }

    /// The farthest target distance at which the motion of `character`'s `key` source counts as
    /// in reach; `None` without the source or a [`MotionReach`].
    pub(in crate::character) fn reach(&self, character: Entity, key: MotionKey) -> Option<f32> {
        let source = self.source_data(character, key)?;
        source.reach.map(|&MotionReach(reach)| reach)
    }

    fn current_motion(&self, character: Entity) -> Option<(Entity, RunningMotion)> {
        let character_data = self.characters.get(character).ok()?;
        let entity = character_data.current_motion?.get();
        let running = self.running_motions.get(entity).ok()?;
        Some((entity, *running))
    }

    fn source_data(
        &self,
        character: Entity,
        key: MotionKey,
    ) -> Option<MotionSourceDataItem<'_, '_>> {
        let character_data = self.characters.get(character).ok()?;
        character_data.sources?.iter().find_map(|source| {
            self.sources
                .get(source)
                .ok()
                .filter(|source_data| *source_data.key == key)
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

    /// Runs the motion of `character`'s `key` source, steering by `target`; without that source it
    /// only stops the running motion. A continuous motion that is already running keeps its
    /// progress and takes the new target; anything else replaces the running motion with a fresh
    /// one.
    pub(in crate::character) fn start(
        &mut self,
        character: Entity,
        key: MotionKey,
        target: Option<Entity>,
    ) {
        let Some(source) = self.status.source_data(character, key) else {
            self.stop(character);
            return;
        };
        let (source, is_finite) = (source.entity, source.is_finite);
        let running = RunningMotion { key, target };
        let current_motion = self.status.current_motion(character);
        if let Some((entity, current)) = current_motion
            && !is_finite
            && current.key == key
        {
            self.commands.entity(entity).try_insert(running);
            return;
        }

        if let Some((entity, _)) = current_motion {
            self.commands.entity(entity).try_despawn();
        }
        // `TunedBy` goes in the same spawn, so `on_motion_started` can read it.
        self.commands.spawn((
            Name::new("Running motion"),
            MotionOf { character },
            running,
            TunedBy { source },
        ));
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
    running_motions: Query<(Entity, &MotionOf, &TunedBy)>,
    finite_sources: Query<(), With<FiniteMotion>>,
    characters: Query<InterruptionState>,
) {
    for (entity, motion_of, tuned_by) in &running_motions {
        if !finite_sources.contains(tuned_by.source) {
            continue;
        }
        let is_interrupted = characters
            .get(motion_of.character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if is_interrupted {
            commands.entity(entity).try_despawn();
        }
    }
}
