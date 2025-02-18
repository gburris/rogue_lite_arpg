//Fucntions that do not modify state in anyway

use bevy::math::{Rect, Vec2};
use bevy_ecs_tilemap::map::TilemapSize;

pub fn calculate_center_rect(map_size: TilemapSize, size: TilemapSize) -> Rect {
    let center = Vec2::new((map_size.x / 2) as f32, (map_size.y / 2) as f32);
    Rect::from_center_size(center, Vec2::new(size.x as f32, size.y as f32))
}

pub fn is_within_bounds(x: i32, y: i32, map_size: TilemapSize) -> bool {
    x >= 0 && y >= 0 && x < map_size.x as i32 && y < map_size.y as i32
}

// src/map/utils/calculations.rs
pub fn calculate_num_dead_zones(area: u32) -> u32 {
    if area < 625 {
        return 0;
    }
    ((area as f32 / 2500.0).ceil() as u32).min(10)
}

pub fn calculate_wall_dimensions(is_horizontal: bool, length: f32) -> (f32, f32) {
    if is_horizontal {
        (length, 1.0)
    } else {
        (1.0, length)
    }
}

pub fn calculate_collider_position(
    start_pos: Vec2,
    width: f32,
    height: f32,
    is_horizontal: bool,
) -> Vec2 {
    if is_horizontal {
        Vec2::new(start_pos.x + (width / 2.0), start_pos.y + 0.5)
    } else {
        Vec2::new(start_pos.x + 0.5, start_pos.y + (height / 2.0))
    }
}
