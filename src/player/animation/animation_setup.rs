use crate::{animation::AnimationTimer, configuration::assets::SpriteAssets, player::Player};

use super::components::{PlayerAnimationConfig, PlayerAnimations};
use bevy::prelude::*;

pub fn set_starting_player_animation_and_sprite_sheet(
    mut commands: Commands,
    animation_config: Res<PlayerAnimationConfig>,
    mut query: Query<Entity, With<Player>>,
    sprites: Res<SpriteAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 13, 21, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let sprite = Sprite::from_atlas_image(
        sprites.player_sprite_sheet.clone(),
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_config
                .get_indices(PlayerAnimations::IdleDown)
                .first,
        },
    );

    if let Ok(player) = query.get_single_mut() {
        commands
            .entity(player)
            .insert(animation_config.get_indices(PlayerAnimations::IdleDown));
        commands.entity(player).insert(AnimationTimer(
            animation_config.get_timer(PlayerAnimations::IdleDown),
        ));
        commands.entity(player).insert(sprite);
    }
}
