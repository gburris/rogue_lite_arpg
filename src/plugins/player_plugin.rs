use bevy::prelude::*;
use crate::systems::{camera_follow_system, draw_cursor, face_cursor_system, player_movement, setup};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (player_movement, face_cursor_system, camera_follow_system, draw_cursor),
        );
    }
}