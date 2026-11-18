use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    DashProgress, Motion, MotionKind, MotionReach, MotionSources, OrbitProgress, Steering,
    SwoopProgress, WanderProgress,
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

/// A character's running motion, on its own entity linked to the character by [`MotionOf`]: the
/// motion, the entity it steers by and the motion source that tunes it. [`MotionCommands`] spawns
/// it with the motion's progress and [`Steering`], and a finite motion despawns it when it ends.
/// Despawning it, however the motion ended, halts the character and, for a finite motion, starts
/// the source's [`Cooldown`] and fires [`MotionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[require(Steering)]
#[component(on_remove = end_running_motion)]
pub(in crate::character) struct RunningMotion {
    pub motion: Motion,
    pub target: Option<Entity>,
    pub source: Option<Entity>,
}

fn end_running_motion(mut world: DeferredWorld, context: HookContext) {
    let Ok((&MotionOf { character }, &running)) = world
        .entity(context.entity)
        .get_components::<(&MotionOf, &RunningMotion)>()
    else {
        return;
    };

    if let Some(mut kinematics) = world.get_mut::<Kinematics>(character) {
        kinematics.stop_moving();
    }

    let kind = running.motion.kind();
    if !kind.is_finite() {
        return;
    }
    let now = world.resource::<Time<Virtual>>().elapsed_secs();
    if let Some(mut cooldown) = running
        .source
        .and_then(|source| world.get_mut::<Cooldown>(source))
    {
        cooldown.start(now);
    }
    world.commands().trigger(MotionFinished {
        entity: character,
        motion: kind,
    });
}

/// A finite motion ended: it completed, lost its target, was interrupted (see
/// [`InterruptionState`]), was replaced or was stopped.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFinished {
    pub entity: Entity,
    pub motion: MotionKind,
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
    kind: &'static MotionKind,
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

    /// Whether a finite motion of `kind` is in progress on `character`. Always false for a
    /// continuous motion.
    pub(in crate::character) fn is_in_progress(&self, character: Entity, kind: MotionKind) -> bool {
        kind.is_finite()
            && self
                .running_motion(character)
                .is_some_and(|running| running.motion.kind() == kind)
    }

    /// Whether `character` can start a motion of `kind` now: it has a source for it if the kind
    /// needs one (see [`MotionKind::needs_source`]), the source's [`Cooldown`] is ready, and
    /// for a finite motion the character is not interrupted (see [`InterruptionState`]).
    pub(in crate::character) fn is_ready(&self, character: Entity, kind: MotionKind) -> bool {
        let is_interrupted = self
            .interruptions
            .get(character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if kind.is_finite() && is_interrupted {
            return false;
        }
        let Some(source) = self.source_data(character, kind) else {
            return !kind.needs_source();
        };
        let now = self.time.elapsed_secs();
        source
            .cooldown
            .is_none_or(|cooldown| cooldown.is_ready(now))
    }

    /// The farthest target distance at which a motion of `kind` counts as in reach for
    /// `character`; `None` without a source or a [`MotionReach`].
    pub(in crate::character) fn reach(&self, character: Entity, kind: MotionKind) -> Option<f32> {
        let source = self.source_data(character, kind)?;
        source.reach.map(|&MotionReach(reach)| reach)
    }

    fn current_motion(&self, character: Entity) -> Option<(Entity, RunningMotion)> {
        let character_data = self.characters.get(character).ok()?;
        let entity = character_data.current_motion?.get();
        let running = self.running_motions.get(entity).ok()?;
        Some((entity, *running))
    }

    fn source_for_kind(&self, character: Entity, kind: MotionKind) -> Option<Entity> {
        self.source_data(character, kind)
            .map(|source| source.entity)
    }

    fn source_data(
        &self,
        character: Entity,
        kind: MotionKind,
    ) -> Option<MotionSourceDataItem<'_, '_>> {
        let character_data = self.characters.get(character).ok()?;
        character_data.sources?.iter().find_map(|source| {
            self.sources
                .get(source)
                .ok()
                .filter(|source_data| *source_data.kind == kind)
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

    /// Runs `motion` on `character`, steering by `target`. A continuous motion of the kind already
    /// running keeps its progress and takes the new settings; anything else replaces the running
    /// motion with a fresh one.
    pub(in crate::character) fn start(
        &mut self,
        character: Entity,
        motion: Motion,
        target: Option<Entity>,
    ) {
        let kind = motion.kind();
        let running = RunningMotion {
            motion,
            target,
            source: self.status.source_for_kind(character, kind),
        };
        let current_motion = self.status.current_motion(character);
        if let Some((entity, current)) = current_motion
            && !kind.is_finite()
            && current.motion.kind() == kind
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
        match kind {
            MotionKind::Dash => {
                running_motion.insert(DashProgress::default());
            }
            MotionKind::Swoop => {
                running_motion.insert(SwoopProgress::default());
            }
            MotionKind::Orbit => {
                running_motion.insert(OrbitProgress::default());
            }
            MotionKind::Wander => {
                running_motion.insert(WanderProgress::default());
            }
            MotionKind::Chase
            | MotionKind::ReturnToHome
            | MotionKind::Flee
            | MotionKind::KeepDistance
            | MotionKind::Idle => {}
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
    running_motions: Query<(Entity, &MotionOf, &RunningMotion)>,
    characters: Query<InterruptionState>,
) {
    for (entity, motion_of, running) in &running_motions {
        if !running.motion.kind().is_finite() {
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
