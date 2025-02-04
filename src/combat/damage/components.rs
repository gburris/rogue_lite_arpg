use std::time::Duration;

use bevy::prelude::*;

//How much damage an enemy does when it collides with you
#[derive(Component, Clone)]
pub struct CollisionDamage {
    pub damage: f32,
}

impl Default for CollisionDamage {
    fn default() -> Self {
        CollisionDamage { damage: 10.1 }
    }
}

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
