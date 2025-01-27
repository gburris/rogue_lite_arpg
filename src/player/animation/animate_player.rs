use crate::{
    animation::{AnimationIndices, AnimationTimer},
    player::{movement::MovementDirection, Player},
};

use super::components::{PlayerAnimationConfig, PlayerAnimations};
use bevy::prelude::*;

pub fn update_player_animation(
    animation_config: Res<PlayerAnimationConfig>,
    //Option cuz single will E X P L O D E when there is no change in movement dir
    //there is no change in movement dir  &  single query resturns 0 results
    update_animation_query: Option<
        Single<
            (
                &mut AnimationIndices,
                &MovementDirection,
                &mut PlayerAnimations,
                &mut AnimationTimer,
                &mut Sprite,
            ),
            (With<Player>, Changed<MovementDirection>),
        >,
    >,
) {
    let Some(query) = update_animation_query else {
        return;
    };
    let (mut indices, movement_direction, mut player_animations, mut timer, mut sprite) =
        query.into_inner();

    let player_animation_from_current_direction =
        PlayerAnimations::from(*movement_direction, *player_animations);

    if *player_animations == player_animation_from_current_direction {
        warn!("We tried to process an animation / direction we already had");
        return;
    }

    *player_animations = player_animation_from_current_direction;
    *indices = animation_config.get_indices(player_animation_from_current_direction);
    *timer = AnimationTimer(animation_config.get_timer(player_animation_from_current_direction));
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = animation_config
            .get_indices(player_animation_from_current_direction)
            .first;
    }
}
