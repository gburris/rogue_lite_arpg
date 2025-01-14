use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets, labels::states::GameState, map::components::Portal,
};

pub fn starting_portal_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    sprites: Res<SpriteAssets>,
) {
    commands.spawn((
        Portal::StartingPortal,
        Sprite::from_image(sprites.run_start_portal.clone()),
        Transform::from_xyz(500.0, 500.0, 1.0),
    ));
    game_state.set(GameState::SpawnPlayer);
}
