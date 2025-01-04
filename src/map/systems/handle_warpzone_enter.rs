use bevy::prelude::*;

use crate::{
    enemy::events::DespawnAllEnemies,
    labels::states::GameState,
    map::events::{DespawnAllPortals, DespawnAllTiles, WarpZoneEnterEvent},
    player::ResetPlayerPosition,
    projectile::events::DespawnAllProjectiles,
};

pub fn handle_warpzone_enter(
    mut commands: Commands,
    mut events: EventReader<WarpZoneEnterEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _event in events.read() {
        commands.trigger(DespawnAllPortals);
        commands.trigger(DespawnAllTiles);
        commands.trigger(DespawnAllEnemies);
        commands.trigger(DespawnAllProjectiles);
        commands.trigger(ResetPlayerPosition);
        game_state.set(GameState::CreateZone);

        // Recolor all the map tiles
        // for (tile_storage, map_size) in tilemap_query.iter_mut() {
        //     for x in 0..map_size.x {
        //         for y in 0..map_size.y {
        //             let tile_pos = TilePos { x, y };
        //             if let Some(tile_entity) = tile_storage.get(&tile_pos) {
        //                 if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
        //                     tile_texture.0 = Level::to_int(&next_level);
        //                 }
        //             }
        //         }
        //     }
        // }
    }
}
