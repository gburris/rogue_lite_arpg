use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;

use crate::world::map::blueprint::TileRect;

//This holds the concept of "Tiles are this big relative to world cordinaties"
#[derive(Resource)]
pub struct MapCoordinates {
    pub tile_size: TilemapTileSize, // Size of each tile in world units
    pub world_origin: Vec2,         // The world space position of the map center
}

//If we want to f with the scale of our tiles:world (e.g. have way more tiles in our world)
//We can edit that here
impl Default for MapCoordinates {
    fn default() -> Self {
        MapCoordinates {
            tile_size: TilemapTileSize::new(32.0, 32.0),
            world_origin: Vec2::ZERO,
        }
    }
}

//Helper impl -> Pass in a tile, and it tells you the world co-ords it maps to
//This seems jank, but it's because the rendering of the tiles has this offset in it's
//Library and in rendering code it's used to "Center" the tiles onto the bevy map
impl MapCoordinates {
    fn units_per_tile(&self) -> Vec2 {
        self.tile_size.into()
    }

    fn bottom_left_offset(&self, map_size_in_tiles: UVec2) -> Vec2 {
        self.units_per_tile() * map_size_in_tiles.as_vec2() * -0.5
    }

    pub fn world_to_tile_pos(&self, map_size_in_tiles: UVec2, world_pos: Vec2) -> Option<UVec2> {
        // Center the world position based on the map size and origin
        let map_position_in_units =
            world_pos - self.world_origin - self.bottom_left_offset(map_size_in_tiles);

        // Convert world position to tile coordinates by dividing by the size of each tile
        let tile_index = map_position_in_units / self.units_per_tile();

        // Check if tile is positive and within map bounds, then convert to UVec2 if valid
        (tile_index.x >= 0.0 && tile_index.y >= 0.0)
            .then(|| tile_index.as_uvec2())
            .filter(|tile_pos| tile_pos.x < map_size_in_tiles.x && tile_pos.y < map_size_in_tiles.y)
    }

    pub fn tile_space_to_world_pos(&self, map_size_in_tiles: UVec2, tile_pos: Vec2) -> Vec2 {
        // Convert tile position to world position by multiplying by the size of each tile
        let world_pos = tile_pos * self.units_per_tile();

        let map_origin = self.world_origin + self.bottom_left_offset(map_size_in_tiles);

        // Center the world position based on the map size and origin
        world_pos + map_origin
    }

    // example: if tile_index is (2,3) and units_per_tile is (32,32), then tile_center is (80,112) (2.5 * 32, 3.5 * 32)
    pub fn tile_center_to_world_pos(&self, map_size_in_tiles: UVec2, tile_index: IVec2) -> Vec2 {
        self.tile_space_to_world_pos(map_size_in_tiles, tile_index.as_vec2() + Vec2::splat(0.5))
    }

    pub fn tile_corner_to_world_pos(&self, map_size_in_tiles: UVec2, tile_index: IVec2) -> Vec2 {
        self.tile_space_to_world_pos(map_size_in_tiles, tile_index.as_vec2())
    }

    pub fn tile_rect_to_world_rect(
        &self,
        map_size_in_tiles: UVec2,
        tile_rect: TileRect,
    ) -> (Vec2, Vec2) {
        let rect_size_in_units = tile_rect.size.as_vec2() * self.units_per_tile();

        // bottom-left corner of the rect in world space
        let min_world = self.tile_corner_to_world_pos(map_size_in_tiles, tile_rect.min);

        // true geometric center
        let center = min_world + rect_size_in_units / 2.0;

        (center, rect_size_in_units)
    }
}
