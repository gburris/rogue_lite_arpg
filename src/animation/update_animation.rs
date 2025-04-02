use crate::{
    ai::state::{ActionState, FacingDirection},
    animation::{AnimationIndices, AnimationTimer, DefaultAnimationConfig},
};

use bevy::prelude::*;

pub fn update_animation(
    animation_config: Res<DefaultAnimationConfig>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &mut AnimationTimer,
            &mut Sprite,
            &ActionState,
            &FacingDirection,
        ),
        Or<(Changed<ActionState>, Changed<FacingDirection>)>,
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
