use avian2d::prelude::Collider;
use bevy::prelude::*;

#[derive(Component)]
#[require(Collider)]
pub struct WarpZone {
    pub level: Level,
}

impl Default for WarpZone {
    fn default() -> Self {
        WarpZone { level: Level::Two }
    }
}

impl WarpZone {
    pub fn to_next_level(current_level: Level) -> Self {
        WarpZone {
            level: Level::next_level(current_level),
        }
    }
}

#[derive(Clone)]
//The levels in the game
pub enum Level {
    One,
    Two,
    Three,
}

impl Level {
    pub fn to_int(&self) -> u32 {
        match self {
            Level::One => 0,
            Level::Two => 1,
            Level::Three => 2,
            // Add more levels as needed
        }
    }
    pub fn next_level(current_level: Level) -> Self {
        match current_level {
            Level::One => Level::Two,
            Level::Two => Level::Three,
            Level::Three => Level::One,
        }
    }
}
