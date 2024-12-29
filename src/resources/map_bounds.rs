use bevy::prelude::Resource;

#[derive(Resource)]
pub struct MapBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}
