use crate::{
    ai::state::{ActionState, FacingDirection},
    animation::{AnimationTimer, DefaultAnimationConfig},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    player::Player,
};

use bevy::prelude::*;

pub fn set_starting_player_animation_and_sprite_sheet(
    mut commands: Commands,
    animation_config: Res<DefaultAnimationConfig>,
    mut query: Query<Entity, With<Player>>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
) {
    let sprite = Sprite::from_atlas_image(
        sprites.player_sprite_sheet.clone(),
        TextureAtlas {
            layout: atlases.player_atlas_layout.clone(),
            index: animation_config
                .get_indices(ActionState::Idle, FacingDirection::Down)
                .first,
        },
    );

    if let Ok(player) = query.get_single_mut() {
        commands.entity(player).insert((
            animation_config.get_indices(ActionState::Idle, FacingDirection::Down),
            AnimationTimer(animation_config.get_timer(ActionState::Idle, FacingDirection::Down)),
            sprite,
        ));
    }
}
