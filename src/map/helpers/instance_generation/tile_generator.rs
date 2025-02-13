use crate::map::components::TileType;
use bevy_ecs_tilemap::map::TilemapSize;

pub fn create_map_with_exterior_walls(map_size: TilemapSize) -> Vec<Vec<TileType>> {
    let mut map = vec![vec![TileType::Ground; map_size.y as usize]; map_size.x as usize];

    // Add top and bottom walls
    for x in 0..map_size.x as usize {
        map[x][0] = TileType::Wall;
        map[x][map_size.y as usize - 1] = TileType::Wall;
    }

    // Add left and right walls
    for y in 0..map_size.y as usize {
        map[0][y] = TileType::Wall;
        map[map_size.x as usize - 1][y] = TileType::Wall;
    }

    map
}
