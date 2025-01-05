use bevy::prelude::*;

#[derive(Resource)]
pub struct MapBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Resource)]
pub struct TileSize {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)]
pub struct CurrentZoneLevel(pub u32);
