// src/components/mod.rs

pub mod player;   // Declare the player module
pub mod position;    // Declare the enemy module
pub mod enemy;

// Re-export the components so they can be easily accessed
pub use player::*;
pub use position::*;
pub use enemy::*;