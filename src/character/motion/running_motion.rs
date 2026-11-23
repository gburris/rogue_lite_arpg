use bevy::ecs::component::Mutable;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    FiniteMotion, MotionKey, MotionSources, SteerInput, SteeredCharacter, Steering, SteeringTargets,
};
use crate::prelude::*;

/// Makes this entity the running motion of `character`. [`MotionCommands`](super::MotionCommands)
/// spawns it as a copy of a motion source, so it carries the source's [`MotionKey`], tuning and
/// [`FiniteMotion`], and links it to the entity it steers by, if any ([`SteeringMotionOf`]).
/// [`on_motion_started`] adds that motion's [`MotionProgress`], and [`apply_motion`] despawns it
/// once a finite motion ends. Despawning it, however the motion ended, halts the character and, for
/// a [`FiniteMotion`], starts the [`Cooldown`] of the character's source with its key and fires
/// [`MotionFinished`].
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

/// A finite motion ended: it completed, lost its target, was interrupted (see
/// [`InterruptionState`]), was replaced or was stopped.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character) struct MotionFinished {
    #[event_target]
    pub character: Entity,
    pub key: MotionKey,
}

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
    world.commands().trigger(MotionFinished { character, key });
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
