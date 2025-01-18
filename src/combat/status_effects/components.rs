use bevy::prelude::*;

use crate::despawn::components::LiveDuration;

use super::events::ApplyStatus;

/**
 * TODO
 */
#[derive(Component, Default)]
#[require(LiveDuration)]
pub struct AppliedStatus;

/**
 * TODO
 */
#[derive(Component, Default)]
pub struct StatusEffect {
    pub duration: f32,
}

/**
 * "Effects" are currently just a list of statuses to apply
 */
#[derive(Component, Default)]
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
pub struct BurningStatus {
    pub damage: f32,
    pub damage_frequency: Timer,
}

impl Default for BurningStatus {
    fn default() -> Self {
        BurningStatus {
            damage: 10.0,
            damage_frequency: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default)]
pub struct FrozenStatus;

#[derive(Component, Clone)]
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
pub struct StunnedStatus;
