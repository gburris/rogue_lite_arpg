use bevy::prelude::*;

// Component for the health bar entity
#[derive(Component)]
pub struct HealthBar {
    pub owner: Entity,
}
