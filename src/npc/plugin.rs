use bevy::prelude::*;

use crate::labels::states::GameState;
use crate::npc::begin_dialogue;
use crate::npc::handle_dialogue_input;
use crate::npc::move_npcs;
use crate::npc::npc_setup;
use crate::npc::update_dialogue_bubbles;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CreateOverworld), npc_setup)
            .add_observer(handle_dialogue_input)
            .add_observer(begin_dialogue)
            .add_systems(Update, update_dialogue_bubbles)
            .add_systems(Update, move_npcs);
    }
}
