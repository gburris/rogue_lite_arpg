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

// System to handle player movement based on movement events
pub fn player_movement(
    mut player_motion_query: Query<
        (&mut MovementDirection, &mut IsMoving, &mut SimpleMotion),
        With<Player>,
    >,
    mut event_reader: EventReader<PlayerMovementEvent>,
) {
    for event in event_reader.read() {
        for (mut movement_direction, mut is_moving, mut motion) in player_motion_query.iter_mut() {
            motion.direction = event.direction;
            *movement_direction = MovementDirection::from_vec2(event.direction);
            is_moving.0 = true;
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
    if *current_player_movement == MovementDirection::None {
        //We are already idle this way, no need to change the animation
        warn!("Player stopped fired while player is stopped");
        return;
    }
    //Slow Timer for idle
    timer.pause();
    *current_player_movement = MovementDirection::None;
    let first_frame = anim_indices.first;
    *anim_indices = AnimationIndices {
        first: first_frame, // 260
        last: first_frame,  // 263
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
