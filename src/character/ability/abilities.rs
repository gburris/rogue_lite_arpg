use bevy::prelude::*;

/// Links an ability entity to the character that uses it.
#[derive(Component, Clone)]
#[relationship(relationship_target = Abilities)]
pub struct AbilityOf {
    pub holder: Entity,
}

/// A character's ability entities, each carrying one capability component and whatever it
/// applies or fires.
#[derive(Component, Clone)]
#[relationship_target(relationship = AbilityOf, linked_spawn)]
pub struct Abilities(Vec<Entity>);
