use bevy::prelude::*;

/**
 * Simple motion has no acceleration and assumes all entities move at max speed unless
 * it is modified by an outside system (such as status effects)
 */
#[derive(Component)]
pub struct SimpleMotion {
    pub direction: Vec2,
    pub max_speed: f32,
    pub current_speed: f32,
    pub can_move: bool, //used for root, stun, anything that stops a moveable entity from moving
}

impl SimpleMotion {
    pub fn new(max_speed: f32) -> Self {
        SimpleMotion {
            direction: Vec2::ZERO,
            current_speed: max_speed,
            max_speed,
            can_move: true,
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
