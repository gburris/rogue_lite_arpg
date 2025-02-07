use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{animation::MovementDirection, movement::components::SimpleMotion};

pub fn simple_movement_to_velocity(mut query: Query<(&SimpleMotion, &mut LinearVelocity)>) {
    for (motion, mut velocity) in query.iter_mut() {
        if motion.is_moving() {
            let temp_vel = motion.get_velocity();
            velocity.x = temp_vel.x;
            velocity.y = temp_vel.y;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

pub fn update_movement_direction_on_motion_change(
    mut query: Query<(&SimpleMotion, &mut MovementDirection), Changed<SimpleMotion>>,
) {
    for (motion, mut movement_direction) in query.iter_mut() {
        movement_direction.set_if_neq(MovementDirection::from_vec2(motion.direction));
    }
}
