use bevy::{ecs::event::EventId, prelude::*, utils::HashSet};

use crate::events::ProjectileHitEvent;

#[derive(Resource)]
pub struct ProcessedProjectiles {
    pub set: HashSet<EventId<ProjectileHitEvent>>,
}
