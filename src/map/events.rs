use bevy::prelude::*;

#[derive(Event)]
pub struct StartRunEvent;

#[derive(Event)]
pub struct WarpZoneEnterEvent {
    pub warp_zone: Entity,
}
