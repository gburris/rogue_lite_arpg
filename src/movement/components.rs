use bevy::prelude::*;

use crate::animation::MovementDirection;

/// Simple motion has no acceleration and assumes all entities move at max speed unless altered by slowed_percentage
/// by Movement
#[derive(Component)]
#[require(MovementDirection)]
pub struct SimpleMotion {
    pub direction: Vec2,
    pub max_speed: f32,
    current_speed: f32,
    /// Applied on top of max_speed, slowed_percentage of 1.0 represents being "stunned"
    slowed_percentage: f32,
}

impl SimpleMotion {
    pub fn new(max_speed: f32) -> Self {
        SimpleMotion {
            max_speed,
            current_speed: max_speed,
            direction: Vec2::ZERO,
            slowed_percentage: 0.0,
        }
    }

    pub fn stun(&mut self) {
        self.slowed_percentage = 1.0;
    }

    pub fn slow(&mut self, percentage: f32) {
        assert!(percentage >= 0.0 && percentage <= 1.0);

        self.slowed_percentage = percentage;
    }

    pub fn remove_debuff(&mut self) {
        self.slowed_percentage = 0.0;
    }

    pub fn start_moving(&mut self, direction: Vec2) {
        self.current_speed = self.max_speed;
        self.direction = direction
    }

    pub fn stop_moving(&mut self) {
        self.current_speed = 0.0;
        self.direction = Vec2::ZERO
    }

    pub fn is_stunned(&self) -> bool {
        self.slowed_percentage >= 1.0
    }

    pub fn is_moving(&self) -> bool {
        self.current_speed > 0.0 && !self.is_stunned()
    }

    pub fn get_velocity(&self) -> Vec2 {
        (self.direction * self.current_speed).clamp_length_max(self.max_speed)
    }
}

impl Default for SimpleMotion {
    fn default() -> Self {
        SimpleMotion::new(10.0)
    }
}
