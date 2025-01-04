use crate::{
    animation::{AnimationIndices, AnimationTimer},
    components::damage_effect::DamageEffect,
    resources::assets::SpriteAssets,
    status_effects::{BurningEffect, FreezingEffect},
};
use avian2d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

use crate::spells::components::Spell;

pub struct SpellFactory;

impl SpellFactory {
    pub fn spawn_spell(
        commands: &mut Commands,
        spell: Spell,
        caster_transform: Transform,
        sprites: &Res<SpriteAssets>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) {
        let spell_speed = 300.0;
        let direction = Mat3::from_quat(caster_transform.rotation).x_axis;
        let velocity = (direction * spell_speed).truncate();

        match spell {
            Spell::Fireball => {
                commands.spawn((
                    spell,
                    caster_transform,
                    DamageEffect { base_damage: 10.0 },
                    LinearVelocity(velocity),
                    BurningEffect {
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        tick_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                        damage_per_second: 5.0,
                    },
                    Sprite::from_image(sprites.fire_bolt.clone()),
                ));
            }
            Spell::Icebolt => {
                let animation_indices = AnimationIndices { first: 0, last: 4 };
                let texture = sprites.ice_bolt.clone();
                let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);

                commands.spawn((
                    spell,
                    DamageEffect { base_damage: 8.0 },
                    LinearVelocity(velocity),
                    FreezingEffect {
                        duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
                        slow_percentage: 0.5,
                    },
                    Sprite::from_atlas_image(
                        texture,
                        TextureAtlas {
                            layout: texture_atlas_layout,
                            index: animation_indices.first,
                        },
                    ),
                    Transform {
                        translation: caster_transform.translation,
                        rotation: caster_transform.rotation,
                        scale: Vec3::splat(2.0),
                    },
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                ));
            }
        }
    }
}
