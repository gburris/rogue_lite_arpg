use bevy::prelude::*;

#[derive(Component, Default)]
pub struct IsMoving(pub bool);

/**
 * Simple motion has no acceleration and assumes all entities move at max speed unless
 * it is modified by an outside system (such as status effects)
 */
#[derive(Component)]
#[require(IsMoving)]
pub struct SimpleMotion {
    pub direction: Vec2,
    pub max_speed: f32,
    pub current_speed: f32,
}

impl SimpleMotion {
    pub fn new(max_speed: f32) -> Self {
        SimpleMotion {
            direction: Vec2::ZERO,
            current_speed: max_speed,
            max_speed,
        }
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
