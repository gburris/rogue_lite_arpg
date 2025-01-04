use bevy::prelude::*;
use bevy::state::commands;
use bevy_ecs_tilemap::map::TilemapId;

use crate::labels::states::GameState;
use crate::map::{RunStartPortal, RunStartPortalEnterEvent};

pub fn handle_run_start_portal_enter(
    mut commands: Commands,
    mut events: EventReader<RunStartPortalEnterEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut tilemap_query: Query<Entity, With<TilemapId>>,
    mut portal_query: Query<Entity, With<RunStartPortal>>,
) {
    for _event in events.read() {
        warn!("Collision with run portal event processing!");
        //All we need to do now is progress the state to BeginRun
        game_state.set(GameState::CreateZone);
        //Clear the map tiles
        for entity in tilemap_query.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }
        //Delete the portal
        for entity in portal_query.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }

        //Delete NPCs
        //Delete projectiles

        debug!("Starting the rouge-like run!");
    }
}
