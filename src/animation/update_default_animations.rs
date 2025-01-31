use crate::animation::{AnimationIndices, AnimationTimer};

use bevy::prelude::*;

use super::{DefaultAnimationConfig, DefaultAnimations, MovementDirection};

pub fn update_animation_on_movement_direction_change(
    animation_config: Res<DefaultAnimationConfig>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &MovementDirection,
            &mut DefaultAnimations,
            &mut AnimationTimer,
            &mut Sprite,
        ),
        (With<DefaultAnimations>, Changed<MovementDirection>),
    >,
) {
    // Using if let to handle the case where we might not have any entities matching the query
    if let Ok((mut indices, movement_direction, mut animations, mut timer, mut sprite)) =
        query.get_single_mut()
    {
        let animation_from_current_direction =
            DefaultAnimations::from(*movement_direction, *animations);

        if *animations == animation_from_current_direction {
            warn!("We tried to process an animation / direction we already had");
            return;
        }

        *animations = animation_from_current_direction;
        *indices = animation_config.get_indices(animation_from_current_direction);
        *timer = AnimationTimer(animation_config.get_timer(animation_from_current_direction));
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = animation_config
                .get_indices(animation_from_current_direction)
                .first;
        }
    }
}
