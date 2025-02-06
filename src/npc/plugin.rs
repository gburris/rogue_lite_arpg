use bevy::prelude::*;

use crate::{labels::sets::InGameSet, npc::move_npcs};

use super::{on_npc_input_interaction, setup::spawn_npcs};

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_npcs)
            .add_observer(on_npc_input_interaction)
            .add_systems(Update, (move_npcs).in_set(InGameSet::Simulation));
    }
}
