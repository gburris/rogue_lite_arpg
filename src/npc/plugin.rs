use bevy::prelude::*;

use crate::{
    labels::sets::InGameSet,
    npc::{begin_dialogue, handle_dialogue_input, move_npcs, update_dialogue_bubbles},
};

use super::setup::spawn_npcs;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_npcs)
            .add_observer(handle_dialogue_input)
            .add_observer(begin_dialogue)
            .add_systems(
                Update,
                (update_dialogue_bubbles, move_npcs).in_set(InGameSet::Simulation),
            );
    }
}
