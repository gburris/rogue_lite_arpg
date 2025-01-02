use bevy::{math::Vec3, prelude::Event};

#[derive(Event)]
pub struct PlayerLevelUpEvent {
    pub new_level: u32,
    pub position: Vec3, // For spawning the level-up effect
}
