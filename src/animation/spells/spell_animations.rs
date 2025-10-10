use bevy::{prelude::*, sprite::Anchor};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    utility::Lifespan,
};

pub fn heal_vfx(
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Sprite {
            image: sprites.tome_of_healing_effect.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.spell_effect.clone(),
                index: 0,
            }),
            ..default()
        },
        Anchor(Vec2::new(0.0, 0.10)),
        AnimationIndices::OneShot(0..=9),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Lifespan::new(1.0),
    )
}
