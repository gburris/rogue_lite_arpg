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

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn new(max_hp: f32) -> Self {
        Self { hp: max_hp, max_hp }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.hp -= amount;
        if self.hp < 0.0 {
            self.hp = 0.0;
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Health {
            hp: 100.0,
            max_hp: 100.0,
        }
    }
}
