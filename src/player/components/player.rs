use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    ai::{state::ActionState, SimpleMotion},
    animation::AnimationTimer,
    combat::Health,
    configuration::YSort,
};

/// How much more experience is required (as a multiplier) after each level up
const PLAYER_LEVEL_REQUIREMENT_MULTIPLIER: f32 = 2.0;

#[derive(Component)]
#[require(
    Health(|| Health::new(100.0)),
    SimpleMotion(|| SimpleMotion::new(350.0)),
    RigidBody,
    LockedAxes(|| LockedAxes::new().lock_rotation()),
    ActionState,
    AnimationTimer,
    YSort
)]
pub struct Player {
    current_level: u32,
    // Outside systems may give the player experience, like when an enemy dies
    pub current_experience: f32,
    next_level_experience_req: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            current_level: 1,
            current_experience: 0.0,
            next_level_experience_req: 20.0,
        }
    }
}

impl Player {
    /// Attempts to increase player level based on current experience and level requirement, and then
    /// sets the new level requirement based on PLAYER_LEVEL_REQUIREMENT_MULTIPLIER
    ///
    /// returns whether the player leveled up
    pub fn attempt_level_up(&mut self) -> bool {
        if self.current_experience >= self.next_level_experience_req {
            self.current_experience -= self.next_level_experience_req;
            self.next_level_experience_req *= PLAYER_LEVEL_REQUIREMENT_MULTIPLIER;
            self.current_level += 1;
            return true;
        }

        false
    }

    pub fn get_progress_to_next_level(&self) -> f32 {
        self.current_experience / self.next_level_experience_req
    }

    pub fn get_level(&self) -> u32 {
        self.current_level
    }
}

#[derive(Component)]
pub struct PlayerCollider;
