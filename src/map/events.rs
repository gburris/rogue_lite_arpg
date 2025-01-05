use bevy::prelude::*;

#[derive(Event)]
pub struct StartRunEvent;

#[derive(Event)]
pub struct WarpZoneEnterEvent;

#[derive(Event)]
pub struct DespawnAllPortals;

#[derive(Event)]
pub struct DespawnAllTiles;
