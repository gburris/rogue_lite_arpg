use bevy::prelude::*;

/// Links a motion source to the character that moves with it. A source is an entity with a
/// [`Motion`](super::Motion) and that motion's tuning, such as a [`Dash`](super::Dash).
#[derive(Component, Clone)]
#[relationship(relationship_target = MotionSources)]
pub struct MotionSourceOf {
    pub character: Entity,
}

/// A character's motion sources, one for each kind of motion it can start that needs one; see
/// [`Motion::needs_source`](super::Motion::needs_source).
#[derive(Component, Clone)]
#[relationship_target(relationship = MotionSourceOf, linked_spawn)]
pub struct MotionSources(Vec<Entity>);

/// The farthest target distance at which an AI character starts a motion of this source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct MotionReach(pub f32);
