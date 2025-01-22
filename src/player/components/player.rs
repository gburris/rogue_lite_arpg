use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{combat::attributes::Health, movement::components::SimpleMotion};

#[derive(Component)]
#[require(Health, SimpleMotion, Collider, PlayerExperience, PlayerLevel)]
pub struct Player;

//Components for experience and leveling
#[derive(Component)]
pub struct PlayerExperience {
    pub current: u32,
    pub next_level_requirement: u32,
}

impl Default for PlayerExperience {
    fn default() -> Self {
        PlayerExperience {
            current: 0,
            next_level_requirement: 20,
        }
    }
}

#[derive(Component)]
pub struct PlayerLevel {
    pub current: u32,
}

impl Default for PlayerLevel {
    fn default() -> Self {
        PlayerLevel { current: 1 }
    }
}
