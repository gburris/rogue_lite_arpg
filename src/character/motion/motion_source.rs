use bevy::prelude::*;

/// Links a motion source to the character that moves with it. A source is an entity with a
/// [`MotionKind`] and that motion's tuning, such as a [`Dash`](super::Dash).
#[derive(Component, Clone)]
#[relationship(relationship_target = MotionSources)]
pub struct MotionSourceOf {
    pub character: Entity,
}

/// A character's motion sources, one for each kind of motion it can start that needs one; see
/// [`MotionKind::needs_source`].
#[derive(Component, Clone)]
#[relationship_target(relationship = MotionSourceOf, linked_spawn)]
pub struct MotionSources(Vec<Entity>);

/// The farthest target distance at which an AI character starts a motion of this source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct MotionReach(pub f32);

/// A [`Motion`](super::Motion) without its settings, for comparing motions and keying what each
/// kind has. On a motion source, it names the motion the source tunes.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKind {
    Wander,
    Chase,
    ReturnToHome,
    Flee,
    KeepDistance,
    Idle,
    Orbit,
    Dash,
    Swoop,
}

impl MotionKind {
    /// Whether this motion ends on its own, rather than running until replaced.
    pub const fn is_finite(self) -> bool {
        match self {
            Self::Dash | Self::Swoop => true,
            Self::Wander
            | Self::Chase
            | Self::ReturnToHome
            | Self::Flee
            | Self::KeepDistance
            | Self::Idle
            | Self::Orbit => false,
        }
    }

    /// Whether a character needs a motion source of this kind to start it, because the motion
    /// reads its tuning from the source.
    pub(super) fn needs_source(self) -> bool {
        match self {
            Self::Wander | Self::Orbit | Self::Dash | Self::Swoop => true,
            Self::Chase | Self::ReturnToHome | Self::Flee | Self::KeepDistance | Self::Idle => {
                false
            }
        }
    }

    /// Whether steering this motion needs a target.
    pub(super) fn needs_target(self) -> bool {
        match self {
            // Flee steers away from the target, so it needs one.
            Self::Chase
            | Self::KeepDistance
            | Self::Orbit
            | Self::Flee
            | Self::Dash
            | Self::Swoop => true,
            Self::Wander | Self::ReturnToHome | Self::Idle => false,
        }
    }

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wander => "Wander",
            Self::Chase => "Chase",
            Self::ReturnToHome => "ReturnToHome",
            Self::Flee => "Flee",
            Self::KeepDistance => "KeepDistance",
            Self::Idle => "Idle",
            Self::Orbit => "Orbit",
            Self::Dash => "Dash",
            Self::Swoop => "Swoop",
        }
    }
}
