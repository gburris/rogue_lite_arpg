use bevy::prelude::*;

use crate::{labels::sets::InGameSet, npc::move_npcs};

use super::{
    dispatch_npc_interaction, on_game_guide_start, on_npc_input_interaction, on_shop_keeper_store_open, on_stat_trainer_store_open, setup::spawn_npcs
};

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_npcs)
            .add_observer(dispatch_npc_interaction)
            .add_observer(on_npc_input_interaction)
            .add_observer(on_shop_keeper_store_open)
            .add_observer(on_stat_trainer_store_open)
            .add_observer(on_game_guide_start)
            .add_systems(Update, (move_npcs).in_set(InGameSet::Simulation));
    }
}
