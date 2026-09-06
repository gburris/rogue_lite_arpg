use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;

use crate::world::map::TileType;

pub struct TileRect {
    pub min: IVec2,
    pub size: UVec2,
}

impl TileRect {
    pub fn new_centered_in(map_size: UVec2, rect_size: UVec2) -> Option<Self> {
        if !rect_size.cmple(map_size).all() {
            return None;
        }

        let min = IVec2::new(
            ((map_size.x - rect_size.x) / 2) as i32,
            ((map_size.y - rect_size.y) / 2) as i32,
        );

        Some(TileRect {
            min,
            size: rect_size,
        })
    }

    pub fn offset_from(&self, offset: IVec2) -> Self {
        TileRect {
            min: self.min + offset,
            size: self.size,
        }
    }

    pub fn max_exclusive(&self) -> IVec2 {
        self.min + self.size.as_ivec2()
    }

    pub fn contains(&self, tile_index: IVec2) -> bool {
        let max_exclusive = self.max_exclusive();
        tile_index.x >= self.min.x
            && tile_index.y >= self.min.y
            && tile_index.x < max_exclusive.x
            && tile_index.y < max_exclusive.y
    }

    pub fn padded(&self, padding: i32) -> Self {
        TileRect {
            min: self.min - IVec2::splat(padding),
            size: self.size + UVec2::splat((padding * 2) as u32),
        }
    }
}

//This holds the concept of "Tiles are this big relative to world cordinaties"
#[derive(Resource)]
pub struct WorldSpaceConfig {
    pub tile_size: TilemapTileSize, // Size of each tile in world units
    pub world_origin: Vec2,         // The world space position of the map center
}

//If we want to f with the scale of our tiles:world (e.g. have way more tiles in our world)
//We can edit that here
impl Default for WorldSpaceConfig {
    fn default() -> Self {
        WorldSpaceConfig {
            tile_size: TilemapTileSize::new(32.0, 32.0),
            world_origin: Vec2::ZERO,
        }
    }
}

//Helper impl -> Pass in a tile, and it tells you the world co-ords it maps to
//This seems jank, but it's because the rendering of the tiles has this offset in it's
//Library and in rendering code it's used to "Center" the tiles onto the bevy map
impl WorldSpaceConfig {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TileGrid {
    map_size: UVec2, // Size of the grid in tiles
    cells: Vec<TileType>,
}

impl TileGrid {
    fn width(&self) -> usize {
        self.map_size.x as usize
    }

    fn height(&self) -> usize {
        self.map_size.y as usize
    }

    fn cell_index(&self, tile_index: IVec2) -> Option<usize> {
        let x = usize::try_from(tile_index.x).ok()?;
        let y = usize::try_from(tile_index.y).ok()?;

        (x < self.width() && y < self.height()).then(|| y * self.width() + x)
    }

    pub fn new(map_size: UVec2, fill: TileType) -> Self {
        TileGrid {
            map_size,
            cells: vec![fill; (map_size.x * map_size.y) as usize],
        }
    }

    pub fn map_size(&self) -> UVec2 {
        self.map_size
    }

    pub fn contains(&self, tile_index: IVec2) -> bool {
        self.cell_index(tile_index).is_some()
    }

    pub fn contains_rect(&self, tile_rect: &TileRect) -> bool {
        let min_index = tile_rect.min;
        let max_index = tile_rect.max_exclusive() - IVec2::new(1, 1);

        self.contains(min_index) && self.contains(max_index)
    }

    pub fn get(&self, x: i32, y: i32) -> Option<TileType> {
        self.cell_index(IVec2::new(x, y))
            .map(|index| self.cells[index])
    }

    pub fn set(&mut self, x: i32, y: i32, tile_type: TileType) -> Option<()> {
        self.cell_index(IVec2::new(x, y))
            .map(|index| self.cells[index] = tile_type)
    }

    pub fn fill_rect(&mut self, tile_rect: &TileRect, tile_type: TileType) -> Option<()> {
        if !self.contains_rect(tile_rect) {
            return None;
        }

        for x in tile_rect.min.x..(tile_rect.min.x + tile_rect.size.x as i32) {
            for y in tile_rect.min.y..(tile_rect.min.y + tile_rect.size.y as i32) {
                self.set(x, y, tile_type);
            }
        }
        Some(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (IVec2, TileType)> + '_ {
        self.cells
            .iter()
            .enumerate()
            .map(move |(index, &tile_type)| {
                let x = index % self.width();
                let y = index / self.width();
                (IVec2::new(x as i32, y as i32), tile_type)
            })
    }

    /// Returns an iterator over the tiles in the specified rectangle, yielding (tile_index, tile_type) pairs.
    pub fn rect_iter(&self, rect: &TileRect) -> impl Iterator<Item = (IVec2, TileType)> + '_ {
        let min = rect.min;
        let max = rect.min + rect.size.as_ivec2();

        (min.x..max.x).flat_map(move |x| {
            (min.y..max.y).filter_map(move |y| {
                self.get(x, y)
                    .map(|tile_type| (IVec2::new(x, y), tile_type))
            })
        })
    }
}
