use bevy::prelude::IVec2;

use crate::world::map::tiles::TileRect;

use super::{
    blueprint::{MapBlueprint, Placement, PlacementKind},
    tiles::TileType,
};

#[derive(Clone, Copy, Debug)]
pub struct Structure {
    pub floor: TileType,
    pub text: &'static str,
}

impl Structure {
    /// Trimmed, non-empty lines of the structure.
    fn rows(&self) -> impl Iterator<Item = &str> + '_ {
        self.text.trim_matches(['\r', '\n']).lines()
    }

    /// Returns the size of the structure: `x` = first-row width (in chars),
    /// `y` = number of lines. Leading/trailing newlines are ignored; all rows
    /// are assumed to have equal width.
    pub fn size(&self) -> IVec2 {
        let mut rows = self.rows().peekable();
        let width = rows.peek().map_or(0, |row| row.chars().count());
        let height = rows.count();
        IVec2::new(
            i32::try_from(width).unwrap_or(i32::MAX),
            i32::try_from(height).unwrap_or(i32::MAX),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureKind {
    HubTown,
    Temple,
}

impl StructureKind {
    pub const fn picture(self) -> &'static Structure {
        match self {
            Self::HubTown => &HUB_TOWN,
            Self::Temple => &TEMPLE,
        }
    }
}

pub static HUB_TOWN: Structure = Structure {
    floor: TileType::Cobblestone,
    text: "
#########################
#########################
##ccccccccccXcccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##cccccccccccccccSccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##cccccHcccccccccTccccc##
##ccccccccccccccccccccc##
##ccccccccccPcccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##ccccccccccccccccccccc##
##########wwwww##########
##########wwwww##########
,,,,,,,,,,wwwww,,,,,,,,,,
,,,,,,,,,,wwwww,,,,,,,,,,
,,,,,,,,,,wwwww,,,,,,,,,,
,,,,,,,,,,wwwww,,,,,,,,,,
,,,,,,,,,,wwwww,,,,,,,,,,
",
};

pub static TEMPLE: Structure = Structure {
    floor: TileType::Cobblestone,
    text: "
#######
#wcccw#
#wcccw#
#wcCcw#
#wcccw#
#wcccw#
##www##
",
};

#[derive(Debug)]
pub enum StructureError {
    Empty,
    TooLarge,
    UnevenRows { row: usize },
    UnknownGlyph { row: usize, col: usize, glyph: char },
    OutOfBounds,
}

pub fn apply_structure(
    blueprint: &mut MapBlueprint,
    structure: &Structure,
    min: IVec2,
) -> Result<(), StructureError> {
    let rows: Vec<&str> = structure.rows().collect();
    let Some(first) = rows.first() else {
        return Err(StructureError::Empty);
    };

    let width = first.chars().count();
    if let Some((r, _)) = rows
        .iter()
        .enumerate()
        .find(|(_, r)| r.chars().count() != width)
    {
        return Err(StructureError::UnevenRows { row: r });
    }

    let (Ok(w), Ok(h)) = (i32::try_from(width), i32::try_from(rows.len())) else {
        return Err(StructureError::TooLarge);
    };

    let size = IVec2::new(w, h);
    let rect = TileRect {
        min,
        size: size.as_uvec2(),
    };

    if !blueprint.tiles.contains_rect(&rect) {
        return Err(StructureError::OutOfBounds);
    }

    let mut parsed = Vec::with_capacity(width.saturating_mul(rows.len()));
    for (row_index, row) in rows.iter().enumerate() {
        for (col, glyph) in row.chars().enumerate() {
            let tile = TileType::from_glyph(glyph);
            let placement = PlacementKind::from_glyph(glyph);
            if glyph != ':' && tile.is_none() && placement.is_none() {
                return Err(StructureError::UnknownGlyph {
                    row: row_index,
                    col,
                    glyph,
                });
            }
            parsed.push((
                IVec2::new(
                    min.x + col as i32,
                    // Row 0 is the top of the map → highest y in the rect.
                    min.y + h - 1 - row_index as i32,
                ),
                tile,
                placement,
            ));
        }
    }

    for (point, tile, placement) in parsed {
        // Bounds verified above, so `set` cannot fail.
        let _ = blueprint.tiles.set(point, tile.unwrap_or(structure.floor));
        if let Some(kind) = placement {
            blueprint.placements.push(Placement { tile: point, kind });
        }
    }
    Ok(())
}
