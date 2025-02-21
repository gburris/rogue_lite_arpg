use bevy::prelude::*;

use crate::{labels::sets::InGameSet, npc::move_npcs};

use super::setup::spawn_npcs;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_npcs)
            .add_systems(Update, (move_npcs).in_set(InGameSet::Simulation));
    }
}
