// src/components/mod.rs

pub mod enemy;
pub mod fireball;
pub mod player; // Declare the player module
pub mod position; // Declare the enemy module

// Re-export the components so they can be easily accessed
pub use enemy::*;
pub use fireball::*;
pub use player::*;
pub use position::*;
