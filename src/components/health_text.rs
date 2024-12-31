use bevy::prelude::*;

// Component for the health text entity
#[derive(Component)]
pub struct HealthText {
    pub owner: Entity,
}
