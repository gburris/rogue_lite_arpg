use bevy::{prelude::*, sprite::Anchor};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::HealingTomeSpellVisualEffect,
};

pub fn on_healing_tome_visual_added(
    trigger: On<Add, HealingTomeSpellVisualEffect>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    let entity = trigger.target();

    commands.entity(entity).insert((
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
    ));
}
