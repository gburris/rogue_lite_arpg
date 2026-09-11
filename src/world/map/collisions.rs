use std::collections::BTreeMap;

use bevy::math::{IVec2, UVec2};

use crate::world::map::blueprint::{TileGrid, TileRect};

pub fn derive_collider_rects(grid: &TileGrid) -> Vec<TileRect> {
    let map_width = i32::try_from(grid.map_size().x).expect("map width fits in i32");
    let map_height = i32::try_from(grid.map_size().y).expect("map height fits in i32");

    let mut completed_collider_rects = Vec::new();
    let mut previous_runs: BTreeMap<(i32, i32), TileRect> = BTreeMap::new();

    for y in 0..map_height {
        let runs = blocking_runs(grid, y, map_width);
        let mut continued_runs = BTreeMap::new();

        for (start_x, end_x) in runs {
            let tree_key = (start_x, end_x);

            let rect = if let Some(mut rect) = previous_runs.remove(&tree_key) {
                rect.size.y += 1;
                rect
            } else {
                TileRect {
                    min: IVec2::new(start_x, y),
                    size: UVec2::new((end_x - start_x) as u32, 1),
                }
            };

            continued_runs.insert(tree_key, rect);
        }

        // Any rectangles that remain in previous_runs did not have a matching run
        // in the current row, so they are complete and can be added to the final list.
        completed_collider_rects.extend(previous_runs.into_values());
        previous_runs = continued_runs;
    }

    completed_collider_rects.extend(previous_runs.into_values());
    completed_collider_rects
}

fn blocking_runs(tiles: &TileGrid, y: i32, map_width: i32) -> Vec<(i32, i32)> {
    let mut runs = Vec::new();
    let mut x = 0;

    while x < map_width {
        while x < map_width
            && !tiles
                .get(x, y)
                .is_some_and(|tile_type| tile_type.blocks_ground())
        {
            x += 1;
        }

        let start = x;

        while x < map_width
            && tiles
                .get(x, y)
                .is_some_and(|tile_type| tile_type.blocks_ground())
        {
            x += 1;
        }

        if start < x {
            runs.push((start, x));
        }
    }

    runs
}
