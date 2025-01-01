use bevy::prelude::*;

// Component for the health bar entity
#[derive(Component)]
pub struct HealthBar {
    pub health_percetange: f32,
}
