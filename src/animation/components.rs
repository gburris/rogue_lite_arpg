use bevy::{prelude::*, utils::HashMap};

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource)]
pub struct DefaultAnimationConfig {
    pub sprite_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animations: HashMap<DefaultAnimations, AnimationData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, States, Hash, Component)]
pub enum DefaultAnimations {
    IdleDown,
    IdleUp,
    IdleRight,
    IdleLeft,
    WalkDown,
    WalkUp,
    WalkRight,
    WalkLeft,
}

impl DefaultAnimations {
    pub fn from(
        direction: MovementDirection,
        player_animations: DefaultAnimations,
    ) -> DefaultAnimations {
        let player_animation_from_current_direction = match direction {
            MovementDirection::Up => DefaultAnimations::WalkUp,
            MovementDirection::Down => DefaultAnimations::WalkDown,
            MovementDirection::Left => DefaultAnimations::WalkLeft,
            MovementDirection::Right => DefaultAnimations::WalkRight,
            MovementDirection::None => {
                // If the player is not moving, map the current walking animation to the corresponding idle animation
                match player_animations {
                    DefaultAnimations::WalkUp => DefaultAnimations::IdleUp,
                    DefaultAnimations::WalkDown => DefaultAnimations::IdleDown,
                    DefaultAnimations::WalkLeft => DefaultAnimations::IdleLeft,
                    DefaultAnimations::WalkRight => DefaultAnimations::IdleRight,
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

impl Default for DefaultAnimationConfig {
    fn default() -> Self {
        let mut animations = HashMap::new();

        // Define all animations
        animations.insert(
            DefaultAnimations::IdleDown,
            AnimationData {
                row: 14,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            DefaultAnimations::IdleUp,
            AnimationData {
                row: 12,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            DefaultAnimations::IdleLeft,
            AnimationData {
                row: 13,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            DefaultAnimations::IdleRight,
            AnimationData {
                row: 15,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );

        animations.insert(
            DefaultAnimations::WalkDown,
            AnimationData {
                row: 10,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            DefaultAnimations::WalkUp,
            AnimationData {
                row: 8,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            DefaultAnimations::WalkLeft,
            AnimationData {
                row: 9,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            DefaultAnimations::WalkRight,
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

impl DefaultAnimationConfig {
    pub fn get_indices(&self, state: DefaultAnimations) -> AnimationIndices {
        let animation_data = self
            .animations
            .get(&state)
            .unwrap_or_else(|| panic!("Missing animation data for {:?}", state));

        let first = animation_data.row * self.columns;
        let last = first + animation_data.frame_count - 1;

        AnimationIndices { first, last }
    }

    pub fn get_timer(&self, state: DefaultAnimations) -> Timer {
        let animation_data = self
            .animations
            .get(&state)
            .unwrap_or_else(|| panic!("Missing animation data for {:?}", state));

        Timer::from_seconds(animation_data.frame_duration, TimerMode::Repeating)
    }
}

#[derive(Component, Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
    #[default]
    None,
}

impl MovementDirection {
    pub fn from_vec2(vec: Vec2) -> Self {
        match vec.normalize() {
            v if v.y > 0.5 => Self::Up,
            v if v.y < -0.5 => Self::Down,
            v if v.x > 0.5 => Self::Right,
            v if v.x < -0.5 => Self::Left,
            _ => Self::None,
        }
    }
}
