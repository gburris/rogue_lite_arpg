use bevy::prelude::*;

#[derive(Event)]
pub struct StartRunEvent;

#[derive(Event)]
pub struct WarpZoneEnterEvent {
    pub warp_zone: Entity,
}

#[derive(Event)]
pub struct DespawnAllPortals;

#[derive(Event)]
pub struct DespawnAllTiles;
