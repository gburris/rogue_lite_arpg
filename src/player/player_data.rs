use bevy::{ecs::system::Resource, prelude::*};
use serde::{Deserialize, Serialize};

use super::interact::InteractionZone;

#[derive(Resource, Deserialize, Serialize, Reflect)]
#[reflect(Resource, Serialize)]
#[allow(non_snake_case)]
pub struct PlayerData {
    /// Starting and ending size of level up ring animation
    pub LEVEL_UP_RING_SIZE: (f32, f32),
    pub MAX_RING_SCALE: f32,
    pub LEVEL_UP_ROTATION_SPEED: f32,
    pub LEVEL_UP_ANIMATION_DURATION: f32,
    pub LEVEL_UP_TEXT_MAX_HEIGHT: f32,
    /// How much more experience is required (as a multiplier) after each level up
    pub PLAYER_LEVEL_REQUIREMENT_MULTIPLIER: f32,
    pub CHARACTER_FEET_POS_OFFSET: f32,
}
pub const CHARACTER_FEET_POS_OFFSET: f32 = -24.0;
pub const LEVEL_UP_RING_SIZE: (f32, f32) = (5.0, 40.0);
pub const MAX_RING_SCALE: f32 = LEVEL_UP_RING_SIZE.1 / LEVEL_UP_RING_SIZE.0;
pub const LEVEL_UP_ROTATION_SPEED: f32 = 2.0;
pub const LEVEL_UP_ANIMATION_DURATION: f32 = 1.2;
pub const LEVEL_UP_TEXT_MAX_HEIGHT: f32 = 100.0;
pub const PLAYER_LEVEL_REQUIREMENT_MULTIPLIER: f32 = 2.0;

impl PlayerData {
    const DEFAULT_PLAYER_DATA: PlayerData = PlayerData {
        LEVEL_UP_RING_SIZE,
        MAX_RING_SCALE,
        LEVEL_UP_ROTATION_SPEED,
        LEVEL_UP_ANIMATION_DURATION,
        LEVEL_UP_TEXT_MAX_HEIGHT,

        CHARACTER_FEET_POS_OFFSET,
        PLAYER_LEVEL_REQUIREMENT_MULTIPLIER,
    };
}
impl Default for PlayerData {
    fn default() -> Self {
        Self::DEFAULT_PLAYER_DATA
    }
}

impl InteractionZone {
    pub const OPEN_CHEST: Self = Self::Square { length: 40.0 };
    pub const NPC: Self = Self::Circle { radius: 30.0 };
    pub const ITEM_PICKUP: Self = Self::Circle { radius: 25.0 };
}
