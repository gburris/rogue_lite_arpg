use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Default, PartialEq, Debug, Hash, Eq, Copy, Clone)]
#[require(FacingDirection, Aim)]
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

    pub fn to_vec2(&self) -> Vec2 {
        match self {
            FacingDirection::Up => Vec2::new(0.0, 1.0),
            FacingDirection::Down => Vec2::new(0.0, -1.0),
            FacingDirection::Left => Vec2::new(-1.0, 0.0),
            FacingDirection::Right => Vec2::new(1.0, 0.0),
        }
    }
}

/// Represents the world coordinate where an entitiy is aiming, for player this is the cursor
#[derive(Component, Default)]
pub struct Aim {
    pub position: Vec2,
    pub target: Option<Entity>,
}

impl Aim {
    pub fn has_target(&self) -> bool {
        self.target.is_some()
    }
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

        // Attacking state take priority over walking / idle, locking facing direction
        if *action_state == ActionState::Attacking {
            continue;
        }

        if motion.is_moving() {
            action_state.set_if_neq(ActionState::Movement);
        } else {
            action_state.set_if_neq(ActionState::Idle);
        }
    }
}
