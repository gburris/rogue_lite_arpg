use bevy::{ecs::system::Resource, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Resource, Deserialize, Serialize, Reflect)]
#[reflect(Resource, Serialize)]
#[allow(non_snake_case)]
pub struct ConfigurationData {
    /// f32::ln(10.0);
    pub DECAY_RATE: f32,
    /// 0.5 is middle of the two positions between the player and the aim position
    pub TARGET_BIAS: f32,
    /// The camera will not go further than this distance from the player
    pub CAMERA_DISTANCE_CONSTRAINT: f32,
}

pub const DECAY_RATE: f32 = 2.3; // f32::ln(10.0);
pub const TARGET_BIAS: f32 = 0.35; // 0.5 is middle of the two positions between the player and the aim position
pub const CAMERA_DISTANCE_CONSTRAINT: f32 = 120.0; // The camera will not go further than this distance from the player
impl ConfigurationData {
    const DEFAULT: ConfigurationData = ConfigurationData {
        DECAY_RATE,
        TARGET_BIAS,
        CAMERA_DISTANCE_CONSTRAINT,
    };
}
impl Default for ConfigurationData {
    fn default() -> Self {
        Self::DEFAULT
    }
}
