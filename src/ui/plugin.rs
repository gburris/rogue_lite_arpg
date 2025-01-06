use crate::{
    labels::{sets::GamePlaySet, states::GameState},
    ui::{game_overlay, pause_menu},
};
use bevy::prelude::*;
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SpawnPlayer), game_overlay::create)
            .add_systems(Update, game_overlay::update.in_set(GamePlaySet::Simulation))
            .add_systems(OnEnter(GameState::AssetLoading), pause_menu::create)
            .add_systems(OnEnter(GameState::Paused), pause_menu::on_pause)
            .add_systems(OnExit(GameState::Paused), pause_menu::on_resume_game)
            .add_systems(
                Update,
                pause_menu::return_to_game.run_if(in_state(GameState::Paused)),
            );
    }
}
