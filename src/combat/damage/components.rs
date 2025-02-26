use std::time::Duration;

use avian2d::prelude::{LayerMask, PhysicsLayer};
use bevy::prelude::*;

use crate::configuration::GameCollisionLayer;

// Component to mark whether an entity has iframes when hit
// Currently only the player has iframes
#[derive(Component)]
pub struct HasIFrames {
    // time to be invulnerable when hit
    pub duration: Duration,
}

// Component to track invulnerability state and timer
#[derive(Component)]
pub struct Invulnerable {
    pub total_time: Timer,
    pub flash_timer: Timer,
    pub is_transparent: bool,
}

impl Invulnerable {
    pub fn new(iframes: &HasIFrames) -> Self {
        Self {
            total_time: Timer::new(iframes.duration, TimerMode::Once),
            ..default()
        }
    }

    pub fn death() -> Self {
        Self {
            total_time: Timer::new(Duration::from_secs(4), TimerMode::Once),
            flash_timer: Timer::new(Duration::from_millis(5000), TimerMode::Repeating), //Don't flash
            is_transparent: false,
        }
    }
}

impl Default for Invulnerable {
    fn default() -> Self {
        Self {
            total_time: Timer::new(Duration::from_secs(2), TimerMode::Once),
            flash_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
            is_transparent: false,
        }
    }
}

#[derive(PartialEq)]
pub enum DamageSource {
    Player,
    Enemy,
    NPC,
    Environment,
}

impl From<DamageSource> for LayerMask {
    fn from(source: DamageSource) -> Self {
        match source {
            DamageSource::Player => GameCollisionLayer::Enemy.to_bits(),
            DamageSource::Enemy => GameCollisionLayer::Player.to_bits(),
            DamageSource::NPC => GameCollisionLayer::Enemy.to_bits(),
            DamageSource::Environment => {
                // Combine both Player and Enemy layers for Environment
                GameCollisionLayer::Enemy.to_bits() | GameCollisionLayer::Player.to_bits()
            }
        }
        .into()
    }
}
