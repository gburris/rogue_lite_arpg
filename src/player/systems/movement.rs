use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    map::resources::MapBounds,
    movement::components::{IsMoving, SimpleMotion},
    player::{
        resources::PlayerSize, Player, PlayerMovementEvent, PlayerStoppedEvent, ResetPlayerPosition,
    },
};

//TODO MOVE TO COMPONENTS ?
//Clone is way wrong here
#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
    None,
}

// System to handle player movement based on movement events
pub fn player_movement(
    mut commands: Commands,
    mut player_motion_query: Query<(&mut IsMoving, &mut SimpleMotion), With<Player>>,
    mut animation_query: Query<
        (
            &mut MovementDirection,
            &mut AnimationTimer,
            &mut AnimationIndices,
            &mut Sprite,
        ),
        With<Player>,
    >,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    for event in event_reader.read() {
        let mut movement_direction = MovementDirection::None;
        for (mut is_moving, mut motion) in player_motion_query.iter_mut() {
            motion.direction = event.direction;
            if motion.direction == Vec2::Y {
                movement_direction = MovementDirection::Up;
            } else if motion.direction == -Vec2::Y {
                movement_direction = MovementDirection::Down;
            } else if motion.direction == Vec2::X {
                movement_direction = MovementDirection::Right;
            } else if motion.direction == -Vec2::X {
                movement_direction = MovementDirection::Left;
            }
            is_moving.0 = true;
        }

        let (mut current_player_movement, mut timer, mut anim_indices, mut sprite) =
            animation_query.single_mut();
        if *current_player_movement == movement_direction {
            //We are already going this way, no need to change the animation
            break;
        } else {
            *current_player_movement = movement_direction;
        }

        let animation_indices = match movement_direction {
            MovementDirection::Up => AnimationIndices {
                first: 8 * 13,        //Row Of Sprite * Numbers of Sprites per row
                last: 8 * 13 + 9 - 1, //Length of this sprits animation
            },
            MovementDirection::Down => AnimationIndices {
                first: 10 * 13,        //Row Of Sprite * Numbers of Sprites per row
                last: 10 * 13 + 9 - 1, //Length of this sprits animation
            },
            MovementDirection::Left => AnimationIndices {
                first: 9 * 13,        //Row Of Sprite * Numbers of Sprites per row
                last: 9 * 13 + 9 - 1, //Length of this sprits animation
            },
            MovementDirection::Right => AnimationIndices {
                first: 11 * 13,        //Row Of Sprite * Numbers of Sprites per row
                last: 11 * 13 + 9 - 1, //Length of this sprits animation
            },
            MovementDirection::None => AnimationIndices {
                first: anim_indices.first,    // 260
                last: anim_indices.first + 9, // 263
            },
        };
        warn!("Insert direction animation {:?}", movement_direction);
        *anim_indices = animation_indices;
        *timer = AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating));
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = anim_indices.first;
        }
    }
}

pub fn on_player_stopped(
    _: Trigger<PlayerStoppedEvent>,
    mut animation_query: Query<
        (
            &mut MovementDirection,
            &mut AnimationTimer,
            &mut AnimationIndices,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    let (mut current_player_movement, mut timer, mut anim_indices, mut sprite) =
        animation_query.single_mut();
    warn!("Insert idle animation");
    //Slow Timer for idle
    *timer = AnimationTimer(Timer::from_seconds(1.0, TimerMode::Repeating));
    *current_player_movement = MovementDirection::None;
    *anim_indices = AnimationIndices {
        first: 20 * 13,        // 260
        last: 20 * 13 + 2 - 1, // 263
    };
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = anim_indices.first;
    }
}
// System to keep player within map bounds
pub fn enforce_map_bounds(
    mut query: Query<&mut Transform, With<Player>>,
    map_bounds: Res<MapBounds>,
    playersize: Res<PlayerSize>,
) {
    for mut transform in query.iter_mut() {
        // Clamp the player position within the map bounds
        transform.translation.x = transform.translation.x.clamp(
            map_bounds.min_x + playersize.x / 2.0,
            map_bounds.max_x - playersize.x / 2.0,
        );
        transform.translation.y = transform.translation.y.clamp(
            map_bounds.min_y + playersize.y / 2.0,
            map_bounds.max_y - playersize.y / 2.0,
        );
    }
}

pub fn reset_player_position(
    _: Trigger<ResetPlayerPosition>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
) {
    if let Ok((mut transform, mut linear_velocity)) = player_query.get_single_mut() {
        // Reset position
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;

        // Reset velocity
        linear_velocity.x = 0.0;
        linear_velocity.y = 0.0;
    }
}
