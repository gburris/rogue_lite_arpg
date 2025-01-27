use bevy::{prelude::*, utils::HashMap};

use crate::{animation::AnimationIndices, player::movement::MovementDirection};

#[derive(Resource)]
pub struct PlayerAnimationConfig {
    pub sprite_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animations: HashMap<PlayerAnimations, AnimationData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, States, Hash, Component)]
pub enum PlayerAnimations {
    IdleDown,
    IdleUp,
    IdleRight,
    IdleLeft,
    WalkDown,
    WalkUp,
    WalkRight,
    WalkLeft,
}

impl PlayerAnimations {
    pub fn from(
        direction: MovementDirection,
        player_animations: PlayerAnimations,
    ) -> PlayerAnimations {
        let player_animation_from_current_direction = match direction {
            MovementDirection::Up => PlayerAnimations::WalkUp,
            MovementDirection::Down => PlayerAnimations::WalkDown,
            MovementDirection::Left => PlayerAnimations::WalkLeft,
            MovementDirection::Right => PlayerAnimations::WalkRight,
            MovementDirection::None => {
                // If the player is not moving, map the current walking animation to the corresponding idle animation
                match player_animations {
                    PlayerAnimations::WalkUp => PlayerAnimations::IdleUp,
                    PlayerAnimations::WalkDown => PlayerAnimations::IdleDown,
                    PlayerAnimations::WalkLeft => PlayerAnimations::IdleLeft,
                    PlayerAnimations::WalkRight => PlayerAnimations::IdleRight,
                    _ => player_animations, // If already idle, keep the current animation
                }
            }
        };
        player_animation_from_current_direction
    }
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub row: usize,          // Row in the sprite sheet
    pub frame_count: usize,  // Number of frames in animation
    pub frame_duration: f32, // Duration of each frame
}

impl Default for PlayerAnimationConfig {
    fn default() -> Self {
        let mut animations = HashMap::new();

        // Define all animations
        animations.insert(
            PlayerAnimations::IdleDown,
            AnimationData {
                row: 14,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            PlayerAnimations::IdleUp,
            AnimationData {
                row: 12,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            PlayerAnimations::IdleLeft,
            AnimationData {
                row: 13,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            PlayerAnimations::IdleRight,
            AnimationData {
                row: 15,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );

        animations.insert(
            PlayerAnimations::WalkDown,
            AnimationData {
                row: 10,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            PlayerAnimations::WalkUp,
            AnimationData {
                row: 8,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            PlayerAnimations::WalkLeft,
            AnimationData {
                row: 9,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            PlayerAnimations::WalkRight,
            AnimationData {
                row: 11,
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

impl PlayerAnimationConfig {
    pub fn get_indices(&self, state: PlayerAnimations) -> AnimationIndices {
        let animation_data = self
            .animations
            .get(&state)
            .unwrap_or_else(|| panic!("Missing animation data for {:?}", state));

        let first = animation_data.row * self.columns;
        let last = first + animation_data.frame_count - 1;

        AnimationIndices { first, last }
    }

    pub fn get_timer(&self, state: PlayerAnimations) -> Timer {
        let animation_data = self
            .animations
            .get(&state)
            .unwrap_or_else(|| panic!("Missing animation data for {:?}", state));

        Timer::from_seconds(animation_data.frame_duration, TimerMode::Repeating)
    }
}
