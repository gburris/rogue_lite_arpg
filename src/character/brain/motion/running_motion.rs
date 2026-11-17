use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{
    Dash, DashProgress, Motion, Orbit, OrbitProgress, Swoop, SwoopProgress, Wander, WanderProgress,
};
use crate::prelude::*;

/// What a motion acts on: a character or object, or a place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::character::brain) enum TargetRef {
    Entity(Entity),
    Point(Vec2),
}

/// The motion a character runs, started through [`MotionCommands`]. Motion systems steer by it. A
/// finite motion also carries its run's progress component, whose removal ends the run and fires
/// [`MotionFinished`]; the motion stays here until the next request.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::character::brain) struct RunningMotion {
    pub motion: Motion,
    pub target: Option<TargetRef>,
}

/// A finite motion's run ended, however it ended: it played out, lost its target, or its
/// character was interrupted (see [`InterruptionState`]) or given another motion.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(in crate::character::brain) struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
}

/// What [`MotionStatus`] reads of a character: the motion it runs and the motion capabilities
/// it carries.
#[derive(QueryData)]
struct MotionHolder {
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
pub(in crate::character::brain) struct MotionStatus<'w, 's> {
    time: Res<'w, Time<Virtual>>,
    characters: Query<'w, 's, MotionHolder>,
}

impl MotionStatus<'_, '_> {
    /// The motion `character` runs; `None` while it runs none.
    pub(in crate::character::brain) fn running_motion(&self, character: Entity) -> Option<Motion> {
        let holder = self.characters.get(character).ok()?;
        holder.running.map(|running| running.motion)
    }

    /// Whether `character` runs a motion of `motion`'s kind that hasn't ended.
    pub(in crate::character::brain) fn is_in_progress(
        &self,
        character: Entity,
        motion: Motion,
    ) -> bool {
        let Ok(holder) = self.characters.get(character) else {
            return false;
        };
        let runs_kind = holder
            .running
            .is_some_and(|running| running.motion.is_same_kind(motion));
        runs_kind
            && match motion {
                Motion::Dash => holder.has_dash_progress,
                Motion::Swoop => holder.has_swoop_progress,
                Motion::Wander
                | Motion::Chase
                | Motion::ReturnToHome
                | Motion::Flee
                | Motion::KeepDistance(_)
                | Motion::Idle
                | Motion::Orbit(_) => true,
            }
    }

    fn has_motion(&self, character: Entity, motion: Motion) -> bool {
        let Ok(holder) = self.characters.get(character) else {
            return false;
        };
        match motion {
            Motion::Dash => holder.dash.is_some(),
            Motion::Swoop => holder.swoop.is_some(),
            Motion::Orbit(_) => holder.has_orbit,
            Motion::Wander => holder.has_wander,
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle => true,
        }
    }

    /// Whether `character` could start `motion` now: it has the capability, and a finite motion is
    /// off cooldown and its character not interrupted (see [`InterruptionState`]).
    pub(in crate::character::brain) fn is_ready(&self, character: Entity, motion: Motion) -> bool {
        let Ok(holder) = self.characters.get(character) else {
            return false;
        };
        let now = self.time.elapsed_secs();
        let cooldown = match motion {
            Motion::Dash => holder.dash.map(Dash::cooldown),
            Motion::Swoop => holder.swoop.map(Swoop::cooldown),
            Motion::Wander
            | Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle
            | Motion::Orbit(_) => return self.has_motion(character, motion),
        };
        cooldown.is_some_and(|cooldown| cooldown.is_ready(now))
            && !holder.interruption.is_interrupted()
    }

    /// The farthest target distance at which `character`'s capability for `motion` counts as in
    /// reach; `None` without the capability or a reach.
    pub(in crate::character::brain) fn reach(
        &self,
        character: Entity,
        motion: Motion,
    ) -> Option<f32> {
        let holder = self.characters.get(character).ok()?;
        match motion {
            Motion::Dash => holder.dash.and_then(Dash::reach),
            Motion::Swoop => holder.swoop.map(Swoop::reach),
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
pub(in crate::character::brain) struct MotionCommands<'w, 's> {
    commands: Commands<'w, 's>,
    status: MotionStatus<'w, 's>,
}

impl<'w, 's> MotionCommands<'w, 's> {
    /// The status these commands check against.
    pub(in crate::character::brain) fn status(&self) -> &MotionStatus<'w, 's> {
        &self.status
    }

    /// Makes `motion` the one `character` runs, steering by `target`. A finite motion starts a
    /// fresh run, and any other motion's run ends. A continuous motion of the kind already running
    /// only takes the new settings, keeping its progress.
    pub(in crate::character::brain) fn start(
        &mut self,
        character: Entity,
        motion: Motion,
        target: Option<TargetRef>,
    ) {
        let keeps_progress = !motion.is_finite()
            && self
                .status
                .running_motion(character)
                .is_some_and(|running| running.is_same_kind(motion));
        let mut entity = self.commands.entity(character);
        entity.try_insert(RunningMotion { motion, target });
        if keeps_progress {
            return;
        }
        // Inserted over an old run, a progress component starts fresh without reporting a finish.
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

    /// Points the motion `character` runs at `target`, keeping its run.
    pub(in crate::character::brain) fn retarget(
        &mut self,
        character: Entity,
        target: Option<TargetRef>,
    ) {
        if let Some(motion) = self.status.running_motion(character) {
            self.commands
                .entity(character)
                .try_insert(RunningMotion { motion, target });
        }
    }

    /// Stops whatever motion `character` runs, ending any run in progress.
    pub(in crate::character::brain) fn stop(&mut self, character: Entity) {
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
