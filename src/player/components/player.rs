use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{attributes::Health, components::AimPosition},
    movement::components::SimpleMotion,
};

#[derive(Component)]
#[require(
    Health(|| Health::new(100.0)),
    SimpleMotion(|| SimpleMotion::new(450.0)),
    PlayerExperience,
    PlayerLevel,
    AimPosition,
    RigidBody,
    LockedAxes(|| LockedAxes::new().lock_rotation())
)]
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
