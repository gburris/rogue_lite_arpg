use crate::{
    animation::{AnimationIndices, AnimationTimer},
    player::{movement::MovementDirection, Player},
};

use super::components::{PlayerAnimationConfig, PlayerAnimations};
use bevy::prelude::*;

pub fn update_player_animation(
    animation_config: Res<PlayerAnimationConfig>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &MovementDirection,
            &mut PlayerAnimations,
            &mut AnimationTimer,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    let (mut indices, movement_direction, mut player_animations, mut timer, mut sprite) =
        query.single_mut();
    let player_animation_from_current_direction = match *movement_direction {
        MovementDirection::Up => PlayerAnimations::WalkUp,
        MovementDirection::Down => PlayerAnimations::WalkDown,
        MovementDirection::Left => PlayerAnimations::WalkLeft,
        MovementDirection::Right => PlayerAnimations::WalkRight,
        MovementDirection::None => PlayerAnimations::IdleDown,
    };

    if *player_animations == player_animation_from_current_direction {
        return;
    }
    *player_animations = player_animation_from_current_direction;
    warn!(
        "Starting a new animation for player {:?}",
        player_animation_from_current_direction
    );
    *indices = animation_config.get_indices(player_animation_from_current_direction);
    *timer = AnimationTimer(animation_config.get_timer(player_animation_from_current_direction));
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = animation_config
            .get_indices(player_animation_from_current_direction)
            .first;
    }
}
