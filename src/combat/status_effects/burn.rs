use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{
        Health,
        damage::{AttemptDamage, Damage},
        status_effects::{Status, StatusApplied, StatusOf},
    },
    prelude::*,
};

#[derive(Component, Clone)]
#[require(Status)]
pub struct Burning {
    pub damage: f32,
    pub damage_frequency: Timer,
}

impl Default for Burning {
    fn default() -> Burning {
        Burning {
            damage: 2.0,
            damage_frequency: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

pub(super) fn tick_burn(mut burn_query: Query<&mut Burning>, time: Res<Time>) {
    for mut burn_status in burn_query.iter_mut() {
        burn_status.damage_frequency.tick(time.delta());
    }
}

// TODO: Modify this to be a "DamagePerSecond" component + system since it isn't specific to burning
pub(super) fn while_burning(
    mut commands: Commands,
    status_query: Query<(&Burning, &StatusOf)>,
    mut health_query: Query<Entity, With<Health>>,
) {
    for (burn, status_of) in status_query.iter() {
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
    status_query: Query<(Entity, &StatusOf, &Lifespan), (With<Burning>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    status_query
        .iter()
        .for_each(|(status, status_of, lifespan)| {
            commands.entity(status).insert(StatusApplied);

            commands.entity(status_of.0).with_child(burn_vfx(
                &sprites,
                &sprite_layouts,
                lifespan.0.clone(),
            ));
        });
}

fn burn_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    duration: Timer,
) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            sprites.flame.clone(),
            TextureAtlas {
                layout: sprite_layouts.flame_vfx.clone(),
                ..default()
            },
        ),
        Transform {
            translation: Vec3::new(
                0.0,
                CHARACTER_FEET_POS_OFFSET + 8.0,
                ZLayer::SpriteForeground.z(),
            ),
            scale: Vec3::new(1.2, 1.2, 1.0),
            ..default()
        },
        AnimationIndices::Cycle((0..=7).cycle()),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Lifespan(duration),
    )
}
