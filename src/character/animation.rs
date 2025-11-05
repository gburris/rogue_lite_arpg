use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    animation::{AnimationData, AnimationIndices, AnimationTimer},
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
        use CharacterAnimationState::*;
        use FacingDirection::*;
        let data = [
            (Idle, Up, (12, 3, 0.5)),
            (Idle, Left, (13, 3, 0.5)),
            (Idle, Down, (14, 3, 0.5)),
            (Idle, Right, (15, 3, 0.5)),
            (Moving, Up, (8, 9, 0.1)),
            (Moving, Left, (9, 9, 0.1)),
            (Moving, Down, (10, 9, 0.1)),
            (Moving, Right, (11, 9, 0.1)),
            (Dying, Up, (20, 5, 0.4)),
            (Dying, Left, (20, 5, 0.4)),
            (Dying, Down, (20, 5, 0.4)),
            (Dying, Right, (20, 5, 0.4)),
            (Attacking, Up, (16, 9, 0.1)),
            (Attacking, Left, (17, 9, 0.1)),
            (Attacking, Down, (18, 9, 0.1)),
            (Attacking, Right, (19, 9, 0.1)),
        ];
        let animations = data
            .into_iter()
            .map(|(state, dir, data)| ((state, dir), AnimationData::from(data)))
            .collect::<HashMap<_, _>>();
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
