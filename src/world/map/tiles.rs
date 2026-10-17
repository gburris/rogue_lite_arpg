use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
        assert!(
            map_size.x > 0 && map_size.y > 0,
            "tile grid dimensions must be non-zero"
        );

        let width = usize::try_from(map_size.x).expect("map width fits usize");
        let height = usize::try_from(map_size.y).expect("map height fits usize");

        let capacity = width
            .checked_mul(height)
            .expect("total tiles in map fits usize");

        TileGrid {
            map_size,
            cells: vec![fill; capacity],
        }
    }

    pub fn map_size(&self) -> UVec2 {
        self.map_size
    }

    pub fn contains(&self, tile_index: IVec2) -> bool {
        self.cell_index(tile_index).is_some()
    }

    pub fn contains_rect(&self, tile_rect: &TileRect) -> bool {
        if tile_rect.size == UVec2::ZERO {
            return tile_rect.min.x >= 0
                && tile_rect.min.y >= 0
                && tile_rect.min.x <= self.width() as i32
                && tile_rect.min.y <= self.height() as i32;
        }
        let min_index = tile_rect.min;
        let Some(max_index) = tile_rect
            .checked_max_exclusive()
            .and_then(|max| max.checked_sub(IVec2::ONE))
        else {
            return false;
        };

        self.contains(min_index) && self.contains(max_index)
    }

    pub fn get(&self, tile_index: IVec2) -> Option<TileType> {
        self.cell_index(tile_index).map(|index| self.cells[index])
    }

    pub fn set(&mut self, tile_index: IVec2, tile_type: TileType) -> bool {
        self.cell_index(tile_index)
            .map(|index| self.cells[index] = tile_type)
            .is_some()
    }

    pub fn fill_rect(&mut self, tile_rect: &TileRect, tile_type: TileType) -> bool {
        if !self.contains_rect(tile_rect) {
            return false;
        }

        let Some(max) = tile_rect.checked_max_exclusive() else {
            return false;
        };
        for x in tile_rect.min.x..max.x {
            for y in tile_rect.min.y..max.y {
                self.set(IVec2::new(x, y), tile_type);
            }
        }
        true
    }

    pub fn fill_border(&mut self, tile_type: TileType) {
        let size = self.map_size;
        if size.x == 0 || size.y == 0 {
            return;
        }

        let w = size.x;
        let h = size.y;

        // Top and bottom rows
        self.fill_rect(
            &TileRect {
                min: IVec2::new(0, 0),
                size: UVec2::new(w, 1),
            },
            tile_type,
        );
        self.fill_rect(
            &TileRect {
                min: IVec2::new(0, h as i32 - 1),
                size: UVec2::new(w, 1),
            },
            tile_type,
        );

        // Left and right columns
        self.fill_rect(
            &TileRect {
                min: IVec2::new(0, 0),
                size: UVec2::new(1, h),
            },
            tile_type,
        );
        self.fill_rect(
            &TileRect {
                min: IVec2::new(w as i32 - 1, 0),
                size: UVec2::new(1, h),
            },
            tile_type,
        );
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

    /// Returns an iterator over the tiles in the specified rectangle, yielding (`tile_index`, `tile_type`) pairs.
    pub fn rect_iter(
        &self,
        rect: &TileRect,
    ) -> Option<impl Iterator<Item = (IVec2, TileType)> + '_> {
        if !self.contains_rect(rect) {
            return None;
        }
        let min = rect.min;
        let max = rect.checked_max_exclusive()?;

        Some((min.x..max.x).flat_map(move |x| {
            (min.y..max.y).filter_map(move |y| {
                self.get(IVec2::new(x, y))
                    .map(|tile_type| (IVec2::new(x, y), tile_type))
            })
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileRect {
    pub min: IVec2,
    pub size: UVec2,
}

impl TileRect {
    pub fn new(min: IVec2, size: UVec2) -> Option<Self> {
        min.checked_add(size.as_ivec2())?;
        Some(Self { min, size })
    }

    pub fn centered_in(map_size: UVec2, rect_size: UVec2) -> Option<Self> {
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

    pub fn translated(&self, offset: IVec2) -> Option<Self> {
        Some(TileRect {
            min: self.min.checked_add(offset)?,
            size: self.size,
        })
    }

    pub fn padded(&self, padding: u32) -> Option<Self> {
        let amount = padding.checked_mul(2)?;
        Some(TileRect {
            min: self
                .min
                .checked_sub(IVec2::splat(i32::try_from(padding).ok()?))?,
            size: self.size.checked_add(UVec2::splat(amount))?,
        })
    }

    pub fn center_tile(&self) -> IVec2 {
        self.min + self.size.as_ivec2() / 2
    }

    pub fn max_exclusive(&self) -> IVec2 {
        self.min + self.size.as_ivec2()
    }

    pub fn checked_max_exclusive(&self) -> Option<IVec2> {
        self.min.checked_add(self.size.as_ivec2())
    }

    pub fn contains(&self, tile_index: IVec2) -> bool {
        let Some(max_exclusive) = self.checked_max_exclusive() else {
            return false;
        };
        tile_index.x >= self.min.x
            && tile_index.y >= self.min.y
            && tile_index.x < max_exclusive.x
            && tile_index.y < max_exclusive.y
    }
}

#[derive(Clone, Eq, Hash, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum TileType {
    Wood,
    Ground,
    Grass,
    Wall,
    Water,
    Cobblestone,
    DeadZone, //Marker for DO NOT RENDER for empty space in the map
}

impl TileType {
    pub const fn ascii_char(self) -> char {
        match self {
            Self::Wood => 'w',
            Self::Ground => '.',
            Self::Grass => ',',
            Self::Wall => '#',
            Self::Water => '~',
            Self::Cobblestone => 'c',
            Self::DeadZone => '_',
        }
    }

    pub const fn visual_key(self) -> char {
        self.ascii_char()
    }

    pub fn blocks_ground(&self) -> bool {
        matches!(self, TileType::Wall | TileType::Water)
    }

    pub fn should_render(&self) -> bool {
        !matches!(self, TileType::DeadZone)
    }

    pub fn can_spawn(&self) -> bool {
        !self.blocks_ground() && self.should_render()
    }
}
