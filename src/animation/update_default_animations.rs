use crate::animation::{AnimationIndices, AnimationTimer};

use bevy::prelude::*;

use super::{DefaultAnimationConfig, DefaultAnimations, MovementDirection};

pub fn update_animation_on_movement_direction_change(
    animation_config: Res<DefaultAnimationConfig>,
    mut animated_query: Query<
        (
            &MovementDirection,
            &mut AnimationIndices,
            &mut DefaultAnimations,
            &mut AnimationTimer,
            &mut Sprite,
        ),
        Changed<MovementDirection>,
    >,
) {
    for (movement_direction, mut indices, mut animations, mut timer, mut sprite) in
        animated_query.iter_mut()
    {
        animations.update_based_on_movement(movement_direction);

        *indices = animation_config.get_indices(&animations);
        *timer = AnimationTimer(animation_config.get_timer(&animations));
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = indices.first;
        }
    }
}
