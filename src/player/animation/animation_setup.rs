use crate::{
    animation::AnimationTimer,
    configuration::assets::{AtlasAssets, SpriteAssets},
    player::Player,
};

use super::components::{PlayerAnimationConfig, PlayerAnimations};
use bevy::prelude::*;

pub fn set_starting_player_animation_and_sprite_sheet(
    mut commands: Commands,
    animation_config: Res<PlayerAnimationConfig>,
    mut query: Query<Entity, With<Player>>,
    sprites: Res<SpriteAssets>,
    atlases: Res<AtlasAssets>,
) {
    let sprite = Sprite::from_atlas_image(
        sprites.player_sprite_sheet.clone(),
        TextureAtlas {
            layout: atlases.player_atlas_layout.clone(),
            index: animation_config
                .get_indices(PlayerAnimations::IdleDown)
                .first,
        },
    );

    if let Ok(player) = query.get_single_mut() {
        commands.entity(player).insert((
            animation_config.get_indices(PlayerAnimations::IdleDown),
            AnimationTimer(animation_config.get_timer(PlayerAnimations::IdleDown)),
            sprite,
        ));
    }
}
