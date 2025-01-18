use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{
        damage::components::DamageEffect,
        spells::components::Spell,
        status_effects::components::{BurningStatus, FrozenStatus, StatusEffect},
    },
    configuration::assets::SpriteAssets,
};

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
                commands
                    .spawn((
                        spell,
                        caster_transform,
                        DamageEffect { base_damage: 10.0 },
                        LinearVelocity(velocity),
                        Sprite::from_image(sprites.fire_bolt.clone()),
                    ))
                    .with_child((StatusEffect { duration: 2.0 }, BurningStatus::default()));
            }
            Spell::Icebolt => {
                let animation_indices = AnimationIndices { first: 0, last: 4 };
                let texture = sprites.ice_bolt.clone();
                let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);

                commands
                    .spawn((
                        spell,
                        DamageEffect { base_damage: 8.0 },
                        LinearVelocity(velocity),
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
                    ))
                    .with_child((StatusEffect { duration: 1.0 }, FrozenStatus::default()));
            }
        }
    }
}
