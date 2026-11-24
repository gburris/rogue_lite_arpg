use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{RunningMotion, RunningMotionOf, SteeringMotionOf};
use crate::prelude::*;

/// Names a motion source, so a brain can pick which of a character's motions to run. The source's
/// tuning component decides how the motion steers and requires its key; a
/// [`Dash`](super::dash::Dash) serves several keys.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKey {
    Idle,
    Wander,
    ReturnToHome,
    Chase,
    Flee,
    KeepDistance,
    Orbit,
    Kite,
    Slide,
    Sidestep,
    StepIn,
    Swoop,
}

impl MotionKey {
    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Wander => "Wander",
            Self::ReturnToHome => "ReturnToHome",
            Self::Chase => "Chase",
            Self::Flee => "Flee",
            Self::KeepDistance => "KeepDistance",
            Self::Orbit => "Orbit",
            Self::Kite => "Kite",
            Self::Slide => "Slide",
            Self::Sidestep => "Sidestep",
            Self::StepIn => "StepIn",
            Self::Swoop => "Swoop",
        }
    }
}

/// Links a motion source to the character that moves with it. A source is an entity with a
/// [`MotionKey`] and that motion's tuning, such as a [`Dash`](super::dash::Dash).
#[derive(Component, Clone)]
#[relationship(relationship_target = MotionSources)]
pub struct MotionSourceOf {
    pub character: Entity,
}

/// A character's motion sources, one for each motion it can run.
#[derive(Component, Clone)]
#[relationship_target(relationship = MotionSourceOf, linked_spawn)]
pub struct MotionSources(Vec<Entity>);

/// Marks a motion source whose motion ends on its own, rather than running until replaced; see
/// [`RunningMotionOf`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct FiniteMotion;

/// Why a character can't start the motion of a source now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::character) enum MotionFailure {
    /// The character has no motion source with the key.
    NoSource,
    /// The motion is finite and the character is interrupted; see [`InterruptionState`].
    CharacterInterrupted,
    /// The motion source's [`Cooldown`] hasn't run out.
    OnCooldown,
}

/// Fired at the character when [`MotionCommands::start`] refuses a motion.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFailed {
    #[event_target]
    pub character: Entity,
    pub key: MotionKey,
}

/// What [`MotionStatus`] reads of a character: its running motion and motion sources.
#[derive(QueryData)]
struct CharacterMotionData {
    running_motion: Option<&'static RunningMotion>,
    sources: Option<&'static MotionSources>,
}

/// What [`MotionStatus`] reads of a running motion.
#[derive(QueryData)]
struct RunningMotionData {
    entity: Entity,
    key: &'static MotionKey,
    steering_motion_of: Option<&'static SteeringMotionOf>,
}

/// What [`MotionStatus`] reads of one motion source.
#[derive(QueryData)]
struct MotionSourceData {
    entity: Entity,
    key: &'static MotionKey,
    is_finite: Has<FiniteMotion>,
    cooldown: Option<&'static Cooldown>,
    reach: Option<&'static Reach>,
}

/// Reads which motion a character runs and which motions it could start.
#[derive(SystemParam)]
pub(in crate::character) struct MotionStatus<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    characters: Query<'w, 's, CharacterMotionData>,
    interruptions: Query<'w, 's, InterruptionState>,
    running_motions: Query<'w, 's, RunningMotionData>,
    sources: Query<'w, 's, MotionSourceData>,
}

impl MotionStatus<'_, '_> {
    /// Whether `character` has a motion source with `key`.
    pub(in crate::character) fn has_source(&self, character: Entity, key: MotionKey) -> bool {
        self.source_data(character, key).is_some()
    }

    /// Whether `character` has a `key` source whose motion ends on its own; see
    /// [`FiniteMotion`].
    pub(in crate::character) fn ends_on_its_own(&self, character: Entity, key: MotionKey) -> bool {
        self.source_data(character, key)
            .is_some_and(|source| source.is_finite)
    }

    /// The motion source `character` would start for `key` now, or why it can't.
    pub(in crate::character) fn can_start(
        &self,
        character: Entity,
        key: MotionKey,
    ) -> Result<Entity, MotionFailure> {
        let source = self
            .source_data(character, key)
            .ok_or(MotionFailure::NoSource)?;
        let is_interrupted = self
            .interruptions
            .get(character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if source.is_finite && is_interrupted {
            return Err(MotionFailure::CharacterInterrupted);
        }
        let now = self.time.elapsed_secs();
        if source
            .cooldown
            .is_some_and(|cooldown| !cooldown.is_ready(now))
        {
            return Err(MotionFailure::OnCooldown);
        }
        Ok(source.entity)
    }

    /// The farthest target distance at which the motion of `character`'s `key` source counts as
    /// in reach; `None` without the source or a [`Reach`].
    pub(in crate::character) fn reach(&self, character: Entity, key: MotionKey) -> Option<f32> {
        let source = self.source_data(character, key)?;
        source.reach.map(|&Reach(reach)| reach)
    }

    fn running_motion(&self, character: Entity) -> Option<RunningMotionDataItem<'_, '_>> {
        let character_data = self.characters.get(character).ok()?;
        self.running_motions
            .get(character_data.running_motion?.get())
            .ok()
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

impl MotionCommands<'_, '_> {
    /// Runs the motion of `character`'s `key` source, steering by `target`. A motion of `key` that
    /// is already running keeps its progress and takes the new target, so calling this every frame
    /// starts the motion once. Otherwise the start needs [`MotionStatus::can_start`]: if allowed,
    /// a fresh copy of the source replaces any running motion; if refused, the running motion
    /// stops and [`MotionFailed`] fires.
    pub(in crate::character) fn start(
        &mut self,
        character: Entity,
        key: MotionKey,
        target: Option<Entity>,
    ) {
        if let Some(running) = self.status.running_motion(character) {
            if *running.key == key {
                let running_target = running
                    .steering_motion_of
                    .map(|steering_motion_of| steering_motion_of.target);
                if running_target != target {
                    let mut motion = self.commands.entity(running.entity);
                    match target {
                        Some(target) => motion.try_insert(SteeringMotionOf { target }),
                        None => motion.try_remove::<SteeringMotionOf>(),
                    };
                }
                return;
            }
            self.commands.entity(running.entity).try_despawn();
        }

        let Ok(source) = self.status.can_start(character, key) else {
            self.commands.trigger(MotionFailed { character, key });
            return;
        };

        // Copies everything the source holds except what describes the source itself.
        let mut source_commands = self.commands.entity(source);
        let mut motion = source_commands.clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<(MotionSourceOf, Cooldown, Reach)>();
        });
        motion.insert((Name::new("Running motion"), RunningMotionOf { character }));
        if let Some(target) = target {
            motion.insert(SteeringMotionOf { target });
        }
    }

    /// Ends `character`'s running motion.
    pub(in crate::character) fn stop(&mut self, character: Entity) {
        if let Some(current) = self.status.running_motion(character) {
            self.commands.entity(current.entity).try_despawn();
        }
    }
}

/// Checks in debug builds that no other source of the new motion source's character has its key,
/// since motions are found by key.
pub(super) fn on_motion_source_added(
    added: On<Add<MotionSourceOf>>,
    sources: Query<(&MotionSourceOf, &MotionKey)>,
    characters: Query<&MotionSources>,
) {
    let Ok((&MotionSourceOf { character }, &key)) = sources.get(added.entity) else {
        return;
    };
    let Ok(character_sources) = characters.get(character) else {
        return;
    };
    // The new source may not be in `MotionSources` yet, so only the others are counted.
    let other_same_key_count = character_sources
        .iter()
        .filter(|&source| source != added.entity)
        .filter(|&source| sources.get(source).is_ok_and(|(_, &other)| other == key))
        .count();
    debug_assert!(
        other_same_key_count == 0,
        "character {character:?} has {other_same_key_count} more motion sources keyed {key:?}; \
         only the first would ever run"
    );
}
