use avian2d::prelude::*;
use bevy::prelude::*;

use crate::movement::components::SimpleMotion;

use super::components::IsMoving;

pub fn simple_movement_to_velocity(
    mut query: Query<(&IsMoving, &SimpleMotion, &mut LinearVelocity)>,
) {
    for (is_moving, motion, mut velocity) in query.iter_mut() {
        if is_moving.0 {
            let temp_vel = motion.get_velocity();
            velocity.x = temp_vel.x;
            velocity.y = temp_vel.y;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}
