use bevy::color::palettes::tailwind::YELLOW_300;
use bevy::prelude::*;

use super::status::{StatusApplied, StatusKind, StatusOf, StatusVfxOf};
use crate::combat::damage::{AttemptDamage, Damage};
use crate::prelude::*;

#[derive(Component, Clone)]
#[require(StatusKind::Burn)]
pub struct Burning {
    damage: f32,
    damage_frequency: Timer,
}

impl Default for Burning {
    fn default() -> Self {
        Self {
            damage: 2.0,
            damage_frequency: Timer::from_seconds(0.7, TimerMode::Repeating),
        }
    }
}

pub(super) fn tick_burn(mut burn_query: Query<&mut Burning, With<StatusApplied>>, time: Res<Time>) {
    for mut burn_status in &mut burn_query {
        burn_status.damage_frequency.tick(time.delta());
    }
}

// TODO: Modify this to be a "DamagePerSecond" component + system since it isn't specific to burning
pub(super) fn while_burning(
    mut commands: Commands,
    status_query: Query<(&Burning, &StatusOf), With<StatusApplied>>,
    mut health_query: Query<Entity, With<Health>>,
) {
    for (burn, status_of) in &status_query {
        if let Ok(entity) = health_query.get_mut(status_of.0)
            && burn.damage_frequency.just_finished()
        {
            commands.trigger(AttemptDamage {
                entity,
                ignore_invulnerable: true,
                damage: Damage::Single(burn.damage),
                ..default()
            });
        }
    }
}

pub(super) fn apply_burning(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Burning>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);

        let target = status_of.0;
        commands.spawn_scene(bsn! {
            @burn_vfx(&sprites, &sprite_layouts, status)
            ChildOf(target)
        });
    }
}

/// Flames licking up around the burning character.
fn burn_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    status: Entity,
) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.flame.clone(),
        TextureAtlas {
            layout: sprite_layouts.flame_vfx.clone(),
            ..default()
        },
    );
    bsn! {
        ~{StatusVfxOf(status)}
        sprite
        Transform {
            translation: Vec3::new(0.0, {CHARACTER_FEET_POS_OFFSET + 8.0}, {ZLayer::SpriteForeground.z()}),
            scale: Vec3::new(1.2, 1.2, 1.0),
        }
        @point_light(Color::from(YELLOW_300), 2.0, 150.0, 10.0)
        ~{AnimationIndices::Cycle((0..=7).cycle())}
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}
