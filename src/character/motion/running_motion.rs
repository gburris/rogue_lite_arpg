use bevy::ecs::component::Mutable;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    FiniteMotion, MotionKey, MotionReach, MotionSourceOf, MotionSources, SteerInput,
    SteeredCharacter, Steering, SteeringTargets,
};
use crate::prelude::*;

/// Makes this entity the running motion of `character`. [`MotionCommands`] spawns it as a copy of
/// a motion source, so it carries the source's [`MotionKey`], tuning and [`FiniteMotion`], and
/// links it to the entity it steers by, if any ([`SteeringMotionOf`]). [`on_motion_started`] adds
/// that motion's [`MotionProgress`], and [`apply_motion`] despawns it once a finite motion ends.
/// Despawning it, however the motion ended, halts the character and, for a [`FiniteMotion`],
/// starts the [`Cooldown`] of the character's source with its key and fires [`MotionFinished`].
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = RunningMotion)]
#[require(Steering)]
#[component(on_remove = end_running_motion)]
pub struct RunningMotionOf {
    pub character: Entity,
}

/// The running motion of this character, if it runs one; see [`RunningMotionOf`].
#[derive(Component, Debug)]
#[relationship_target(relationship = RunningMotionOf, linked_spawn)]
pub struct RunningMotion(Entity);

impl RunningMotion {
    pub fn get(&self) -> Entity {
        self.0
    }
}

/// Makes this running motion steer by `target`, such as a `LocationHint`.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = SteeringMotions)]
pub(super) struct SteeringMotionOf {
    pub target: Entity,
}

/// The running motions that steer by this entity; see [`SteeringMotionOf`].
#[derive(Component, Debug)]
#[relationship_target(relationship = SteeringMotionOf)]
pub(super) struct SteeringMotions(Vec<Entity>);

/// A motion's state while it runs, which [`on_motion_started`] adds to its running motion and
/// [`apply_motion`] advances.
pub(super) trait MotionProgress: Component<Mutability = Mutable> + Default {
    /// The tuning component of the running motions that get this progress.
    type Tuning: Component;

    /// Advances the motion by one frame of `input`.
    fn step(&mut self, tuning: &Self::Tuning, input: &SteerInput) -> MotionStep;
}

/// The result of advancing a motion by one frame.
pub(super) enum MotionStep {
    Steer(Steering),
    /// The motion ended this frame.
    Finished,
}

/// Gives each newly started running motion with a `Progress::Tuning` a fresh `Progress`.
pub(super) fn on_motion_started<Progress: MotionProgress>(
    added: On<Add<RunningMotionOf>>,
    mut commands: Commands,
    tunings: Query<(), With<Progress::Tuning>>,
) {
    if tunings.contains(added.entity) {
        commands.entity(added.entity).insert(Progress::default());
    }
}

/// Advances each running motion that has a `Progress`, and despawns it once its step finishes or
/// its character can't steer.
pub(super) fn apply_motion<Progress: MotionProgress>(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    mut running_motions: Query<(
        Entity,
        &RunningMotionOf,
        &Progress::Tuning,
        Option<&SteeringMotionOf>,
        &mut Progress,
        &mut Steering,
    )>,
) {
    for (entity, running_motion_of, tuning, steering_motion_of, mut progress, mut steering) in
        &mut running_motions
    {
        let step = match characters.get(running_motion_of.character) {
            Ok(character) => {
                let input = character.build_steer_input(steering_motion_of, &targets, &time);
                progress.step(tuning, &input)
            }
            Err(_) => MotionStep::Finished,
        };
        match step {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished => {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

fn end_running_motion(mut world: DeferredWorld, context: HookContext) {
    let running_motion = world.entity(context.entity);
    let Ok((&RunningMotionOf { character }, &key)) =
        running_motion.get_components::<(&RunningMotionOf, &MotionKey)>()
    else {
        return;
    };
    let is_finite = running_motion.contains::<FiniteMotion>();

    if let Some(mut kinematics) = world.get_mut::<Kinematics>(character) {
        kinematics.stop_moving();
    }
    if !is_finite {
        return;
    }

    // Started before `MotionFinished`, so a brain deciding on that finish sees the cooldown.
    let source = world.get::<MotionSources>(character).and_then(|sources| {
        sources
            .iter()
            .find(|&source| world.get::<MotionKey>(source) == Some(&key))
    });
    let now = world.resource::<Time<Virtual>>().elapsed_secs();
    if let Some(source) = source
        && let Some(mut cooldown) = world.get_mut::<Cooldown>(source)
    {
        cooldown.start(now);
    }
    world.commands().trigger(MotionFinished {
        entity: character,
        key,
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
    reach: Option<&'static MotionReach>,
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
    /// Runs the motion of `character`'s `key` source, steering by `target`; without that source it
    /// only stops the running motion. A motion of `key` that is already running keeps its progress
    /// and takes the new target, so calling this every frame starts the motion once; any other
    /// running motion is replaced with a fresh copy of the source.
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
        let source = source.entity;

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

        // Copies everything the source holds except what describes the source itself.
        let mut source_commands = self.commands.entity(source);
        let mut motion = source_commands.clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<(MotionSourceOf, Cooldown, MotionReach)>();
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

/// Ends each finite motion whose character is interrupted; see [`InterruptionState`].
pub(super) fn end_interrupted_motions(
    mut commands: Commands,
    running_motions: Query<(Entity, &RunningMotionOf), With<FiniteMotion>>,
    characters: Query<InterruptionState>,
) {
    for (entity, running_motion_of) in &running_motions {
        let is_interrupted = characters
            .get(running_motion_of.character)
            .is_ok_and(|interruption| interruption.is_interrupted());
        if is_interrupted {
            commands.entity(entity).try_despawn();
        }
    }
}
