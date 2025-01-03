use bevy::prelude::{Component, Timer};

#[derive(Component)]
pub struct FreezingEffect {
    pub duration: Timer,
    pub slow_percentage: f32,
}
