use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::FacingDirection, combat::components::ActionState, movement::components::SimpleMotion,
};

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

pub fn update_facing_direction_and_action_state_on_motion_change(
    mut query: Query<
        (&SimpleMotion, &mut ActionState, &mut FacingDirection),
        Changed<SimpleMotion>,
    >,
) {
    for (motion, mut action_state, mut facing_direction) in query.iter_mut() {
        facing_direction.set_if_neq(FacingDirection::from_vec2(
            &facing_direction,
            motion.direction,
        ));

        //Defeated and Attacking state take priority over walking / idle
        if *action_state == ActionState::Attacking || *action_state == ActionState::Defeated {
            continue;
        } else {
            if motion.is_moving() {
                action_state.set_if_neq(ActionState::Movement);
            } else {
                action_state.set_if_neq(ActionState::Idle);
            }
        }
    }
}
