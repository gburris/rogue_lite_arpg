use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A row-major grid of tile data with a bottom-left origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileGrid<T = TileKind> {
    map_size: UVec2, // Size of the grid in tiles
    cells: Vec<T>,
}

impl<T: Copy> TileGrid<T> {
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

    /// Creates a grid filled with `fill`.
    pub fn new(map_size: UVec2, fill: T) -> Self
    where
        T: Clone,
    {
        assert!(
            map_size.x > 0 && map_size.y > 0,
            "tile grid dimensions must be non-zero"
        );
        let width = usize::try_from(map_size.x).expect("map width fits usize");
        let height = usize::try_from(map_size.y).expect("map height fits usize");
        let capacity = width
            .checked_mul(height)
            .expect("total tiles in map fits usize");
        Self {
            map_size,
            cells: vec![fill; capacity],
        }
    }

    /// Returns the grid dimensions in tiles.
    pub fn map_size(&self) -> UVec2 {
        self.map_size
    }

    /// Returns whether the grid contains the given tile coordinate.
    pub fn contains(&self, tile_index: IVec2) -> bool {
        self.cell_index(tile_index).is_some()
    }

    /// Returns whether the entire rectangle lies within the grid.
    pub fn contains_rect(&self, tile_rect: &TileRect) -> bool {
        let Some(max_index) = tile_rect
            .checked_max_exclusive()
            .and_then(|max| max.checked_sub(IVec2::ONE))
        else {
            return false;
        };
        self.contains(tile_rect.min) && self.contains(max_index)
    }

    /// Returns the tile value at a coordinate, or `None` when out of bounds.
    pub fn get(&self, tile_index: IVec2) -> Option<T> {
        self.cell_index(tile_index).map(|index| self.cells[index])
    }

    /// Sets a tile value; returns `false` when the coordinate is out of bounds.
    pub fn set(&mut self, tile_index: IVec2, value: T) -> bool {
        self.cell_index(tile_index)
            .map(|index| self.cells[index] = value)
            .is_some()
    }

    /// Fills the rectangle; returns `false` when it is not fully in bounds.
    pub fn fill_rect(&mut self, tile_rect: &TileRect, value: T) -> bool {
        if !self.contains_rect(tile_rect) {
            return false;
        }
        let Some(max) = tile_rect.checked_max_exclusive() else {
            return false;
        };
        for x in tile_rect.min.x..max.x {
            for y in tile_rect.min.y..max.y {
                self.set(IVec2::new(x, y), value);
            }
        }
        true
    }

    /// Fills the grid's outermost rows and columns.
    pub fn fill_border(&mut self, value: T) {
        let size = self.map_size;
        self.fill_rect(
            &TileRect {
                min: IVec2::ZERO,
                size: UVec2::new(size.x, 1),
            },
            value,
        );
        self.fill_rect(
            &TileRect {
                min: IVec2::new(0, size.y as i32 - 1),
                size: UVec2::new(size.x, 1),
            },
            value,
        );
        self.fill_rect(
            &TileRect {
                min: IVec2::ZERO,
                size: UVec2::new(1, size.y),
            },
            value,
        );
        self.fill_rect(
            &TileRect {
                min: IVec2::new(size.x as i32 - 1, 0),
                size: UVec2::new(1, size.y),
            },
            value,
        );
    }

    /// Iterates over all tiles in row-major order with their coordinates.
    pub fn iter(&self) -> impl Iterator<Item = (IVec2, T)> + '_ {
        self.cells.iter().enumerate().map(move |(index, &value)| {
            (
                IVec2::new((index % self.width()) as i32, (index / self.width()) as i32),
                value,
            )
        })
    }

    /// Iterates over tiles in a rectangle, or returns `None` if it is out of bounds.
    pub fn rect_iter(&self, rect: &TileRect) -> Option<impl Iterator<Item = (IVec2, T)> + '_> {
        if !self.contains_rect(rect) {
            return None;
        }
        let min = rect.min;
        let max = rect.checked_max_exclusive()?;
        Some((min.x..max.x).flat_map(move |x| {
            (min.y..max.y).filter_map(move |y| {
                self.get(IVec2::new(x, y))
                    .map(|value| (IVec2::new(x, y), value))
            })
        }))
    }
}

/// An axis-aligned rectangle of tiles with an exclusive upper bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileRect {
    /// Bottom-left tile coordinate.
    pub min: IVec2,
    /// Rectangle dimensions in tiles.
    pub size: UVec2,
}

impl TileRect {
    /// Creates a rectangle if its exclusive upper bound is representable.
    pub fn new(min: IVec2, size: UVec2) -> Option<Self> {
        min.checked_add(size.as_ivec2())?;
        Some(Self { min, size })
    }

    /// Creates a rectangle centered in a map, if it fits.
    pub fn centered_in(map_size: UVec2, rect_size: UVec2) -> Option<Self> {
        if !rect_size.cmple(map_size).all() {
            return None;
        }

        let min = IVec2::new(
            ((map_size.x - rect_size.x) / 2) as i32,
            ((map_size.y - rect_size.y) / 2) as i32,
        );

        Some(Self {
            min,
            size: rect_size,
        })
    }

    /// Returns the rectangle translated by an offset, or `None` on overflow.
    pub fn translated(&self, offset: IVec2) -> Option<Self> {
        Some(Self {
            min: self.min.checked_add(offset)?,
            size: self.size,
        })
    }

    /// Returns the tile at the rectangle's integer midpoint.
    pub fn center_tile(&self) -> IVec2 {
        self.min + self.size.as_ivec2() / 2
    }

    /// Returns the exclusive upper bound.
    pub fn max_exclusive(&self) -> IVec2 {
        self.min + self.size.as_ivec2()
    }

    /// Returns the exclusive upper bound, or `None` on overflow.
    pub fn checked_max_exclusive(&self) -> Option<IVec2> {
        self.min.checked_add(self.size.as_ivec2())
    }

    /// Returns whether the rectangle contains the tile coordinate.
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

/// A tile material or special map marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Wood,
    Ground,
    Grass,
    Wall,
    Water,
    Cobblestone,
    DeadZone, // Marker for DO NOT RENDER for empty space in the map
}

impl TileKind {
    /// Every tile kind. `from_glyph` searches this list, so a new kind must be added here
    /// as well as to `glyph`.
    pub const ALL: [Self; 7] = [
        Self::Wood,
        Self::Ground,
        Self::Grass,
        Self::Wall,
        Self::Water,
        Self::Cobblestone,
        Self::DeadZone,
    ];

    /// Returns the glyph used to encode this tile.
    pub const fn glyph(self) -> char {
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

    /// Parses a tile glyph, returning `None` for unrecognized characters.
    pub const fn from_glyph(glyph: char) -> Option<Self> {
        let mut index = 0;
        while index < Self::ALL.len() {
            if Self::ALL[index].glyph() == glyph {
                return Some(Self::ALL[index]);
            }
            index += 1;
        }
        None
    }

    /// Returns whether this tile blocks ground placement.
    pub fn blocks_ground(self) -> bool {
        matches!(self, Self::Wall | Self::Water | Self::DeadZone)
    }

    /// Returns whether this tile should be rendered.
    pub fn should_render(self) -> bool {
        !matches!(self, Self::DeadZone)
    }

    /// Returns whether this tile can host a spawn.
    pub fn can_spawn(self) -> bool {
        !self.blocks_ground() && self.should_render()
    }
}
