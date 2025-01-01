use crate::systems::{
    animate_sprite, camera_follow_system, cast_spell_system, draw_cursor, face_cursor_system,
    player_movement, player_setup,
};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Player Plugin! added!");
        app.add_systems(Startup, player_setup).add_systems(
            Update,
            (
                player_movement,
                face_cursor_system,
                camera_follow_system,
                draw_cursor,
                cast_spell_system,
                animate_sprite,
            ),
        );
    }
}
