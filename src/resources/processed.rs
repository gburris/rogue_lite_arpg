use bevy::{ecs::event::EventId, prelude::*, utils::HashSet};

use crate::events::ProjectileHitEvent;
use crate::events::WarpZoneEnterEvent;

#[derive(Resource)]
pub struct ProcessedProjectiles {
    pub set: HashSet<EventId<ProjectileHitEvent>>,
}
#[derive(Resource)]
pub struct ProcessedWarpZoneEvents {
    pub set: HashSet<EventId<WarpZoneEnterEvent>>,
}
