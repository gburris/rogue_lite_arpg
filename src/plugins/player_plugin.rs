use crate::systems::{
    camera_follow_system, draw_cursor, face_cursor_system, player_movement, setup,
};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Player Plugin! added!");
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                player_movement,
                face_cursor_system,
                camera_follow_system,
                draw_cursor,
            ),
        );
    }
}
