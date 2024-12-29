use bevy::prelude::Resource;

#[derive(Resource)]
pub struct TileSize {
    pub x: f32,
    pub y: f32,
}
