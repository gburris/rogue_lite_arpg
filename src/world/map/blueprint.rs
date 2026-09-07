use bevy::prelude::*;

use crate::{
    prelude::{EnemyType, NpcType},
    world::map::TileType,
};

#[derive(Resource, Debug, PartialEq, Eq)]
pub struct MapBlueprint {
    pub tiles: TileGrid,
    pub placements: Vec<Placement>,
    pub rng_seed: u64,
}

impl MapBlueprint {
    pub fn new(map_size: UVec2, floor: TileType, rng_seed: u64) -> Self {
        let tiles = TileGrid::new(map_size, floor);

        Self {
            tiles,
            rng_seed,
            placements: Vec::new(),
        }
    }

    pub fn map_size(&self) -> UVec2 {
        self.tiles.map_size()
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

    pub fn padded(&self, padding: i32) -> Self {
        TileRect {
            min: self.min - IVec2::splat(padding),
            size: self.size + UVec2::splat((padding * 2) as u32),
        }
    }

    pub fn center(&self) -> IVec2 {
        self.min + self.size.as_ivec2() / 2
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
}

#[derive(Debug, PartialEq, Eq)]
pub struct Placement {
    pub tile: IVec2,
    pub kind: PlacementType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlacementType {
    PlayerSpawn,
    Enemy(EnemyType),
    Npc(NpcType),
    Chest,
    LevelExit,
}
