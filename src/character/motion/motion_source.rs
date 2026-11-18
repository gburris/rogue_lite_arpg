use bevy::prelude::*;

use crate::prelude::*;

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

/// How long a motion source waits after a motion of it ends before it can start again.
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct MotionCooldown(Cooldown);

impl MotionCooldown {
    /// Waits `seconds` after each motion ends.
    pub fn new(seconds: f32) -> Self {
        Self(Cooldown::new(seconds))
    }
}

/// The farthest target distance at which an AI character starts a motion of this source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct MotionReach(pub f32);
