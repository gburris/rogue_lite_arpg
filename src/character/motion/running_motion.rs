use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    DashProgress, Motion, MotionCooldown, MotionKind, MotionReach, MotionSources, OrbitProgress,
    Steering, SwoopProgress, WanderProgress,
};
use crate::prelude::*;

/// The motion a character runs, the entity it steers by and the motion source that tunes it, set
/// through [`MotionCommands`]. A finite motion also has a progress component; removing it ends the
/// motion and fires [`MotionFinished`]. A finite motion that ends removes this component with its
/// progress. Steering drives only characters with this component, and removing it halts the
/// character.
#[derive(Component, Clone, Copy, Debug)]
#[require(Steering)]
#[component(on_remove = halt_character)]
pub(in crate::character) struct RunningMotion {
    pub motion: Motion,
    pub target: Option<Entity>,
    pub source: Option<Entity>,
}

fn halt_character(mut world: DeferredWorld, context: HookContext) {
    if let Some(mut kinematics) = world.get_mut::<Kinematics>(context.entity) {
        kinematics.stop_moving();
    }
}

/// Starts the [`MotionCooldown`] of the source of `character`'s running motion, if it has one.
pub(super) fn start_motion_source_cooldown(world: &mut DeferredWorld, character: Entity) {
    let now = world.resource::<Time<Virtual>>().elapsed_secs();
    let Some(source) = world
        .get::<RunningMotion>(character)
        .and_then(|running| running.source)
    else {
        return;
    };
    if let Some(mut cooldown) = world.get_mut::<MotionCooldown>(source) {
        cooldown.start(now);
    }
}

/// A finite motion ended: it completed, lost its target, was interrupted (see
/// [`InterruptionState`]), or was replaced.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFinished {
    pub entity: Entity,
    pub motion: MotionKind,
}

/// What [`MotionStatus`] reads of a character: its running motion and motion sources.
#[derive(QueryData)]
struct CharacterMotionData {
    running: Option<&'static RunningMotion>,
    sources: Option<&'static MotionSources>,
    has_dash_progress: Has<DashProgress>,
    has_swoop_progress: Has<SwoopProgress>,
    interruption: InterruptionState,
}

/// What [`MotionStatus`] reads of one motion source.
#[derive(QueryData)]
struct MotionSourceData {
    entity: Entity,
    kind: &'static MotionKind,
    cooldown: Option<&'static MotionCooldown>,
    reach: Option<&'static MotionReach>,
}

/// Reads which motion a character runs and which motions it could start.
#[derive(SystemParam)]
pub(in crate::character) struct MotionStatus<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    characters: Query<'w, 's, CharacterMotionData>,
    sources: Query<'w, 's, MotionSourceData>,
}

impl MotionStatus<'_, '_> {
    /// The motion `character` runs and its target; `None` while it runs none.
    pub(in crate::character) fn running_motion(&self, character: Entity) -> Option<RunningMotion> {
        let character_data = self.characters.get(character).ok()?;
        character_data.running.copied()
    }

    /// Whether a finite motion of `kind` is in progress on `character`. Always false for a
    /// continuous motion.
    pub(in crate::character) fn is_in_progress(&self, character: Entity, kind: MotionKind) -> bool {
        let Ok(character_data) = self.characters.get(character) else {
            return false;
        };
        match kind {
            MotionKind::Dash => character_data.has_dash_progress,
            MotionKind::Swoop => character_data.has_swoop_progress,
            MotionKind::Wander
            | MotionKind::Chase
            | MotionKind::ReturnToHome
            | MotionKind::Flee
            | MotionKind::KeepDistance
            | MotionKind::Idle
            | MotionKind::Orbit => false,
        }
    }

    /// Whether `character` can start a motion of `kind` now: it has a source for it if the kind
    /// needs one (see [`MotionKind::needs_source`]), the source's [`MotionCooldown`] is ready, and
    /// for a finite motion the character is not interrupted (see [`InterruptionState`]).
    pub(in crate::character) fn is_ready(&self, character: Entity, kind: MotionKind) -> bool {
        let Ok(character_data) = self.characters.get(character) else {
            return false;
        };
        if kind.is_finite() && character_data.interruption.is_interrupted() {
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

    /// `character`'s motion source of `kind`, if it has one.
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

    /// Sets `motion` as `character`'s running motion, steering by `target`, and ends any other
    /// motion's progress. A finite motion starts with fresh progress. A continuous motion of the
    /// kind already running keeps its progress.
    pub(in crate::character) fn start(
        &mut self,
        character: Entity,
        motion: Motion,
        target: Option<Entity>,
    ) {
        let kind = motion.kind();
        let keeps_progress = !kind.is_finite()
            && self
                .status
                .running_motion(character)
                .is_some_and(|running| running.motion.kind() == kind);
        let source = self.status.source_for_kind(character, kind);

        let mut entity = self.commands.entity(character);
        if !keeps_progress {
            // Before the new running motion, so a replaced finite motion's hooks read its own.
            entity.try_remove::<(DashProgress, SwoopProgress, OrbitProgress, WanderProgress)>();
        }
        entity.try_insert(RunningMotion {
            motion,
            target,
            source,
        });
        if keeps_progress {
            return;
        }

        match kind {
            MotionKind::Dash => {
                entity.try_insert(DashProgress::default());
            }
            MotionKind::Swoop => {
                entity.try_insert(SwoopProgress::default());
            }
            MotionKind::Orbit => {
                entity.try_insert(OrbitProgress::default());
            }
            MotionKind::Wander => {
                entity.try_insert(WanderProgress::default());
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
        if let Some(running) = self.status.running_motion(character) {
            self.commands
                .entity(character)
                .try_insert(RunningMotion { target, ..running });
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
                .try_remove::<(RunningMotion, DashProgress, SwoopProgress)>();
        }
    }
}
