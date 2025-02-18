use std::ops::Range;

use crate::map::components::TileType;

use bevy_ecs_tilemap::map::TilemapSize;

use super::map_data::MapData;

pub fn add_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {
    add_horizontal_exterior_walls(map_data, map_size);
    add_vertical_exterior_walls(map_data, map_size);
}

fn add_horizontal_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {

    add_wall_section(
        map_data,
        true,
        0..map_size.x as usize, 
        0,                     
        0,                     
    );

    add_wall_section(
        map_data,
        true,
        0..map_size.x as usize,  
        map_size.y as usize - 1,
        map_size.y - 1,        
    );
}

fn add_vertical_exterior_walls(map_data: &mut MapData, map_size: TilemapSize) {

    add_wall_section(
        map_data,
        false,
        0..map_size.y as usize, 
        0,                     
        0,                      
    );

    add_wall_section(
        map_data,
        false,
        0..map_size.y as usize,
        map_size.x as usize - 1, 
        map_size.x - 1,         
    );
}

fn add_wall_section(
    map_data: &mut MapData,
    is_horizontal: bool,
    range: Range<usize>,
    position: usize,
    offset: u32,
) {
    let mut wall_start = 0;
    let mut current_length = 0;

    for i in range {
        let (x, y) = if is_horizontal {
            (i, position)
        } else {
            (position, i)
        };

        if map_data.tiles[x][y] != TileType::DeadZone {
            map_data.tiles[x][y] = TileType::Wall;
            if current_length == 0 {
                wall_start = i;
            }
            current_length += 1;
        } else if current_length > 0 {
            map_data.add_wall_collider(
                if is_horizontal {
                    (wall_start as u32, offset)
                } else {
                    (offset, wall_start as u32)
                },
                is_horizontal,
                current_length as u32,
            );
            current_length = 0;
        }
    }

    if current_length > 0 {
        map_data.add_wall_collider(
            if is_horizontal {
                (wall_start as u32, offset)
            } else {
                (offset, wall_start as u32)
            },
            is_horizontal,
            current_length as u32,
        );
    }
}
