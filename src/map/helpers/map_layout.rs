use bevy_ecs_tilemap::map::TilemapSize;
use rand::Rng;

use crate::map::components::{TileType, WallSection};

pub fn generate_map_layout(map_size: TilemapSize) -> Vec<Vec<TileType>> {
    let mut rng = rand::thread_rng();
    let mut map = vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize];

    // Reduce number of walls to about 2% of map size
    let num_walls = (map_size.x as f32 * map_size.y as f32 * 0.002) as i32;

    for _ in 0..num_walls {
        // Increased maximum wall length for more interesting structures
        let wall_length = rng.gen_range(10..25); // Random length between 10-25 tiles

        // Add some padding from the edges
        let start_x = rng.gen_range(5..(map_size.x as i32 - wall_length));
        let start_y = rng.gen_range(5..(map_size.y as i32 - wall_length));

        // Randomly choose horizontal or vertical wall
        let is_horizontal = rng.gen_bool(0.5);

        // Only place wall if area is clear (to prevent overcrowding)
        let can_place = |x: i32, y: i32| -> bool {
            let padding = 3; // Space between walls
            for dx in -padding..=padding {
                for dy in -padding..=padding {
                    let check_x = x + dx;
                    let check_y = y + dy;
                    if check_x >= 0
                        && check_x < map_size.x as i32
                        && check_y >= 0
                        && check_y < map_size.y as i32
                        && map[check_x as usize][check_y as usize] == TileType::Wall
                    {
                        return false;
                    }
                }
            }
            true
        };

        // Only place wall if entire length is clear
        let mut can_place_wall = true;
        for i in 0..wall_length {
            let (check_x, check_y) = if is_horizontal {
                (start_x + i, start_y)
            } else {
                (start_x, start_y + i)
            };

            if !can_place(check_x, check_y) {
                can_place_wall = false;
                break;
            }
        }

        // Place wall if area is clear
        if can_place_wall {
            for i in 0..wall_length {
                if is_horizontal {
                    map[(start_x + i) as usize][start_y as usize] = TileType::Wall;
                } else {
                    map[start_x as usize][(start_y + i) as usize] = TileType::Wall;
                }
            }
        }
    }

    map
}

pub fn find_wall_sections(map_layout: &[Vec<TileType>], map_size: TilemapSize) -> Vec<WallSection> {
    let mut visited = vec![vec![false; map_size.y as usize]; map_size.x as usize];
    let mut wall_sections = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            if visited[x as usize][y as usize]
                || map_layout[x as usize][y as usize] != TileType::Wall
            {
                continue;
            }

            // Check horizontal wall
            if x + 1 < map_size.x && map_layout[(x + 1) as usize][y as usize] == TileType::Wall {
                let mut section = WallSection::new((x, y), true);
                let mut current_x = x + 1;

                visited[x as usize][y as usize] = true;

                while current_x < map_size.x
                    && map_layout[current_x as usize][y as usize] == TileType::Wall
                {
                    visited[current_x as usize][y as usize] = true;
                    section.extend((current_x, y));
                    current_x += 1;
                }

                wall_sections.push(section);
            }
            // Check vertical wall
            else if y + 1 < map_size.y
                && map_layout[x as usize][(y + 1) as usize] == TileType::Wall
            {
                let mut section = WallSection::new((x, y), false);
                let mut current_y = y + 1;

                visited[x as usize][y as usize] = true;

                while current_y < map_size.y
                    && map_layout[x as usize][current_y as usize] == TileType::Wall
                {
                    visited[x as usize][current_y as usize] = true;
                    section.extend((x, current_y));
                    current_y += 1;
                }

                wall_sections.push(section);
            }
            // Single wall tile
            else {
                visited[x as usize][y as usize] = true;
                wall_sections.push(WallSection::new((x, y), true));
            }
        }
    }

    wall_sections
}
