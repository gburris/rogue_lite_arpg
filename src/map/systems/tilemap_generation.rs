use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::resources::{assets::SpriteAssets, CurrentZoneLevel, MapBounds, TileSize};

pub fn generate_tilemap(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    mapbounds: Res<MapBounds>,
    tilesize: Res<TileSize>,
    zone_level: Res<CurrentZoneLevel>,
) {
    let texture_handle: Handle<Image> = sprites.tiles.clone();
    // Size of the tile map in tiles.
    let map_size: TilemapSize = TilemapSize {
        x: ((mapbounds.max_x - mapbounds.min_x) / tilesize.x) as u32, // 3200/16 = 200 tiles
        y: ((mapbounds.max_y - mapbounds.min_y) / tilesize.y) as u32,
    };

    // To create a map we use the TileStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = TileStorage::empty(map_size);

    // For the purposes of this example, we consider a tilemap with rectangular tiles.
    let map_type = TilemapType::Square;

    // Create a tilemap entity a little early
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    let tilemap_entity = commands.spawn_empty().id();

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };

            // Currently have 6 colors in tiles.png, alternating through them as we go down levels
            let color = zone_level.0 % 6;
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(color),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }
    // This is the size of each individual tiles in pixels.
    let tile_size = TilemapTileSize {
        x: tilesize.x,
        y: tilesize.y,
    };
    let grid_size = tile_size.into();

    // Spawns a tilemap.
    // Once the tile storage is inserted onto the tilemap entity it can no longer be accessed.
    commands.entity(tilemap_entity).insert((TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        map_type,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    },));
}
