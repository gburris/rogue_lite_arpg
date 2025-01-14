use bevy::prelude::Resource;

#[derive(Resource)]
pub struct PlayerSize {
    pub x: f32,
    pub y: f32,
}
