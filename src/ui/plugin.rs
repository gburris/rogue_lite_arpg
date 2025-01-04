use crate::{
    labels::{sets::GamePlaySet, states::GameState},
    ui::{setup::setup_ui, update},
};
use bevy::prelude::*;
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SpawnPlayer), setup_ui)
            .add_systems(Update, update.in_set(GamePlaySet::Simulation));
    }
}
