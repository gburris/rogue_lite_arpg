use bevy::prelude::*;

/// Names a motion source, so a brain can pick which of a character's motions to run. The source's
/// tuning component, such as a [`Dash`](super::Dash), decides how the motion steers, so one kind
/// of tuning can sit under several keys.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKey {
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
/// [`MotionKey`] and that motion's tuning, such as a [`Dash`](super::Dash).
#[derive(Component, Clone)]
#[relationship(relationship_target = MotionSources)]
pub struct MotionSourceOf {
    pub character: Entity,
}

/// A character's motion sources, one for each motion it can run.
#[derive(Component, Clone)]
#[relationship_target(relationship = MotionSourceOf, linked_spawn)]
pub struct MotionSources(Vec<Entity>);

/// Marks a motion source whose motion ends on its own, rather than running until replaced. Its end
/// starts the source's [`Cooldown`](crate::prelude::Cooldown) and reports
/// [`MotionFinished`](super::MotionFinished).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct FiniteMotion;

/// Links a running motion to the motion source whose tuning it reads.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = TunedMotion)]
pub(super) struct TunedBy {
    pub source: Entity,
}

/// The running motion this motion source tunes, if one runs; see [`TunedBy`].
#[derive(Component, Debug)]
#[relationship_target(relationship = TunedBy)]
pub(super) struct TunedMotion(Entity);

/// The farthest target distance at which an AI character starts a motion of this source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct MotionReach(pub f32);
