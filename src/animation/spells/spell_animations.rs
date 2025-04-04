use bevy::{prelude::*, sprite::Anchor};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::HealingTomeSpellVisualEffect,
};

pub fn on_healing_tome_visual_added(
    trigger: Trigger<OnAdd, HealingTomeSpellVisualEffect>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    layouts: Res<SpriteSheetLayouts>,
) {
    let entity = trigger.entity();

    commands.entity(entity).insert((
        Sprite {
            image: sprites.tome_of_healing_effect.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: layouts.spell_effect.clone(),
                index: 0,
            }),
            anchor: Anchor::Custom(Vec2::new(0.0, 0.10)),
            ..default()
        },
        AnimationIndices::OneShot(0..=9),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
