use bevy::prelude::*;

use crate::{
    labels::sets::InGameSet,
    npc::{begin_dialogue, handle_dialogue_input, move_npcs, npc_setup, update_dialogue_bubbles},
};

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(npc_setup)
            .add_observer(handle_dialogue_input)
            .add_observer(begin_dialogue)
            .add_systems(
                Update,
                (update_dialogue_bubbles, move_npcs).in_set(InGameSet::Simulation),
            );
    }
}
