use bevy::prelude::*;

use crate::ai::SimpleMotion;

#[derive(Component, Default, PartialEq, Debug, Hash, Eq, Copy, Clone)]
pub enum ActionState {
    Attacking,
    Defeated, //Death Animation
    Movement,
    #[default]
    Idle,
    Casting,
}

/// The direction a character faces and aims are not tied to each other in this game
#[derive(Component, Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FacingDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl FacingDirection {
    pub fn from_vec2(&self, vec: Vec2) -> Self {
        match vec.normalize() {
            v if v.y > 0.5 => Self::Up,
            v if v.y < -0.5 => Self::Down,
            v if v.x > 0.5 => Self::Right,
            v if v.x < -0.5 => Self::Left,
            _ => *self,
        }
    }
}

/// Represents the world coordinate where an entitiy is aiming, for player this is the cursor
#[derive(Component, Default)]
pub struct AimPosition {
    pub position: Vec2,
}

pub fn update_state_on_simple_motion_change(
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

        // Defeated and Attacking state take priority over walking / idle
        if *action_state == ActionState::Attacking || *action_state == ActionState::Defeated {
            continue;
        }

        if motion.is_moving() {
            action_state.set_if_neq(ActionState::Movement);
        } else {
            action_state.set_if_neq(ActionState::Idle);
        }
    }
}
