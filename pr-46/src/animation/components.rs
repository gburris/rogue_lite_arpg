use bevy::{prelude::*, utils::HashMap};

use crate::ai::state::{ActionState, FacingDirection};

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
    pub is_one_shot: bool,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource)]
pub struct DefaultAnimationConfig {
    pub sprite_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animations: HashMap<(ActionState, FacingDirection), AnimationData>,
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub row: usize,          // Row in the sprite sheet
    pub frame_count: usize,  // Number of frames in animation
    pub frame_duration: f32, // Duration of each frame
}

impl Default for DefaultAnimationConfig {
    fn default() -> Self {
        let mut animations = HashMap::new();

        // Define all animations
        animations.insert(
            (ActionState::Idle, FacingDirection::Down),
            AnimationData {
                row: 14,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (ActionState::Idle, FacingDirection::Up),
            AnimationData {
                row: 12,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (ActionState::Idle, FacingDirection::Left),
            AnimationData {
                row: 13,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (ActionState::Idle, FacingDirection::Right),
            AnimationData {
                row: 15,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Down),
            AnimationData {
                row: 10,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Up),
            AnimationData {
                row: 8,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Left),
            AnimationData {
                row: 9,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Right),
            AnimationData {
                row: 11,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );
        //Literally less code to repeat this 4x than solve it in a proper way
        //All four FacingDirections map to defeated down row / animation
        animations.insert(
            (ActionState::Defeated, FacingDirection::Down),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (ActionState::Defeated, FacingDirection::Left),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );
        animations.insert(
            (ActionState::Defeated, FacingDirection::Right),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (ActionState::Defeated, FacingDirection::Up),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (ActionState::Attacking, FacingDirection::Up),
            AnimationData {
                row: 16,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Attacking, FacingDirection::Down),
            AnimationData {
                row: 18,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );
        animations.insert(
            (ActionState::Attacking, FacingDirection::Left),
            AnimationData {
                row: 17,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Attacking, FacingDirection::Right),
            AnimationData {
                row: 19,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        // Add more animations as needed...

        Self {
            sprite_size: UVec2::new(64, 64),
            columns: 13,
            rows: 21,
            animations,
        }
    }
}

impl DefaultAnimationConfig {
    pub fn get_animation(&self, state: ActionState, direction: FacingDirection) -> &AnimationData {
        self.animations.get(&(state, direction)).unwrap_or_else(|| {
            panic!(
                "Missing animation data for {:?} {:?}",
                state.clone(),
                direction.clone()
            )
        })
    }

    pub fn get_indices(&self, state: ActionState, direction: FacingDirection) -> AnimationIndices {
        let animation = self.get_animation(state, direction);
        let first = animation.row * self.columns;
        AnimationIndices {
            first,
            last: first + animation.frame_count - 1,
            is_one_shot: false,
        }
    }

    pub fn get_timer(&self, state: ActionState, direction: FacingDirection) -> Timer {
        let animation = self.get_animation(state, direction);
        Timer::from_seconds(animation.frame_duration, TimerMode::Repeating)
    }
}
