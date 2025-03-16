use bevy::{prelude::*, sprite::Anchor};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{HealingTomeSpellVisualEffect, ShieldSpellVisualEffect},
    labels::layer::ZLayer,
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
            image: sprites.tome_of_healing_effect_sprite_sheet.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: layouts.spell_effect.clone(),
                index: 0,
            }),
            anchor: Anchor::Custom(Vec2::new(0.0, 0.10)),
            ..default()
        },
        AnimationIndices {
            is_one_shot: true,
            first: 0,
            last: 9,
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

pub fn on_shield_effect_added(
    trigger: Trigger<OnAdd, ShieldSpellVisualEffect>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    layouts: Res<SpriteSheetLayouts>,
) {
    let entity = trigger.entity();

    commands.entity(entity).insert((
        Sprite {
            image: sprites.shield_effect_sprite_sheet.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: layouts.spell_effect.clone(),
                index: 0,
            }),
            anchor: Anchor::Custom(Vec2::new(0.0, 0.10)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, ZLayer::WeaponBehindSprite.z()),
        AnimationIndices {
            is_one_shot: false,
            first: 2,
            last: 7,
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
