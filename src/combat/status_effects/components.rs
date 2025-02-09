use bevy::prelude::*;

use crate::despawn::components::LiveDuration;

use super::events::ApplyStatus;

/**
 * "Effects" are currently just a list of statuses to apply
 */
#[derive(Component, Default, Clone)]
pub struct EffectsList {
    pub effects: Vec<ApplyStatus>,
}

#[derive(Clone)]
pub enum StatusType {
    Burning(BurningStatus),
    Frozen,
    Slowed(SlowedStatus),
    Stunned,
}

#[derive(Component, Clone)]
#[require(Status)]
pub struct BurningStatus {
    pub damage: f32,
    pub damage_frequency: Timer,
}

impl Default for BurningStatus {
    fn default() -> Self {
        BurningStatus {
            damage: 2.0,
            damage_frequency: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default)]
#[require(Status)]
pub struct FrozenStatus;

#[derive(Component, Clone)]
#[require(Status)]
pub struct SlowedStatus {
    pub slow_percentage: f32,
}

impl Default for SlowedStatus {
    fn default() -> Self {
        SlowedStatus {
            slow_percentage: 0.5,
        }
    }
}

#[derive(Component, Default)]
#[require(Status)]
pub struct StunnedStatus;

/**
 * Simple marker component we can use to f
 */
#[derive(Component, Default)]
#[require(LiveDuration)]
pub struct Status;
