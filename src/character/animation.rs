use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    animation::{AnimationData, AnimationIndices, AnimationTimer},
    combat::Health,
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_animation_state, cycle_character_animation)
            .chain()
            .in_set(InGameSystems::Vfx),
    )
    .insert_resource(DefaultAnimationConfig::default());
}

#[derive(Component, Default, PartialEq, Debug, Hash, Eq, Copy, Clone)]
#[require(FacingDirection, SimpleMotion)]
pub enum CharacterAnimationState {
    #[default]
    Idle,
    Moving,
    Attacking,
    Dying,
}

fn update_animation_state(
    mut character_query: Query<
        (
            &SimpleMotion,
            &AttackState,
            Option<&Health>,
            &mut CharacterAnimationState,
            &mut FacingDirection,
        ),
        Or<(Changed<SimpleMotion>, Changed<AttackState>, Changed<Health>)>,
    >,
) {
    for (motion, attack_state, health, mut animation_state, mut facing_direction) in
        character_query.iter_mut()
    {
        facing_direction.set_if_neq(FacingDirection::from_vec2(
            &facing_direction,
            motion.direction,
        ));

        if health.is_none() || health.is_some_and(|h| h.hp <= 0.0) {
            animation_state.set_if_neq(CharacterAnimationState::Dying);
            continue;
        }

        // Attacking animation takes priority over walking / idle
        if attack_state.is_attacking {
            animation_state.set_if_neq(CharacterAnimationState::Attacking);
            continue;
        }

        if motion.is_moving() {
            animation_state.set_if_neq(CharacterAnimationState::Moving);
        } else {
            animation_state.set_if_neq(CharacterAnimationState::Idle);
        }
    }
}

fn cycle_character_animation(
    animation_config: Res<DefaultAnimationConfig>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &mut AnimationTimer,
            &mut Sprite,
            &CharacterAnimationState,
            &FacingDirection,
        ),
        Or<(Changed<CharacterAnimationState>, Changed<FacingDirection>)>,
    >,
) {
    for (mut indices, mut timer, mut sprite, state, direction) in query.iter_mut() {
        *indices = animation_config.get_indices(*state, *direction);
        *timer = AnimationTimer(animation_config.get_timer(*state, *direction));
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = indices.start();
        }
    }
}

#[derive(Resource)]
pub struct DefaultAnimationConfig {
    pub columns: usize,
    pub animations: HashMap<(CharacterAnimationState, FacingDirection), AnimationData>,
}

impl Default for DefaultAnimationConfig {
    fn default() -> Self {
        let mut animations = HashMap::default();

        // Define all animations
        animations.insert(
            (CharacterAnimationState::Idle, FacingDirection::Down),
            AnimationData {
                row: 14,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (CharacterAnimationState::Idle, FacingDirection::Up),
            AnimationData {
                row: 12,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (CharacterAnimationState::Idle, FacingDirection::Left),
            AnimationData {
                row: 13,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (CharacterAnimationState::Idle, FacingDirection::Right),
            AnimationData {
                row: 15,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );

        animations.insert(
            (CharacterAnimationState::Moving, FacingDirection::Down),
            AnimationData {
                row: 10,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (CharacterAnimationState::Moving, FacingDirection::Up),
            AnimationData {
                row: 8,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (CharacterAnimationState::Moving, FacingDirection::Left),
            AnimationData {
                row: 9,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (CharacterAnimationState::Moving, FacingDirection::Right),
            AnimationData {
                row: 11,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );
        //Literally less code to repeat this 4x than solve it in a proper way
        //All four FacingDirections map to defeated down row / animation
        animations.insert(
            (CharacterAnimationState::Dying, FacingDirection::Down),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (CharacterAnimationState::Dying, FacingDirection::Left),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );
        animations.insert(
            (CharacterAnimationState::Dying, FacingDirection::Right),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (CharacterAnimationState::Dying, FacingDirection::Up),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (CharacterAnimationState::Attacking, FacingDirection::Up),
            AnimationData {
                row: 16,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (CharacterAnimationState::Attacking, FacingDirection::Down),
            AnimationData {
                row: 18,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );
        animations.insert(
            (CharacterAnimationState::Attacking, FacingDirection::Left),
            AnimationData {
                row: 17,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (CharacterAnimationState::Attacking, FacingDirection::Right),
            AnimationData {
                row: 19,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        // Add more animations as needed...

        Self {
            columns: 13,
            animations,
        }
    }
}

impl DefaultAnimationConfig {
    pub fn get_animation(
        &self,
        state: CharacterAnimationState,
        direction: FacingDirection,
    ) -> &AnimationData {
        self.animations.get(&(state, direction)).unwrap_or_else(|| {
            panic!(
                "Missing animation data for {:?} {:?}",
                state.clone(),
                direction.clone()
            )
        })
    }

    pub fn get_indices(
        &self,
        state: CharacterAnimationState,
        direction: FacingDirection,
    ) -> AnimationIndices {
        let animation = self.get_animation(state, direction);
        let first = animation.row * self.columns;
        let last = first + animation.frame_count - 1;
        AnimationIndices::Cycle((first..=last).cycle())
    }

    pub fn get_timer(&self, state: CharacterAnimationState, direction: FacingDirection) -> Timer {
        let animation = self.get_animation(state, direction);
        Timer::from_seconds(animation.frame_duration, TimerMode::Repeating)
    }
}
