use std::thread::spawn;

use avian2d::prelude::*;
use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{
        damage::components::CollisionDamage,
        projectile::on_damage_dealt::on_damage_dealt_despawn,
        spells::components::Spell,
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
        weapon::projectile_weapon::ProjectileWeapon,
    },
    configuration::assets::SpriteAssets,
    player::systems::AimPosition,
};

/// For this simple example, we will just organize our systems
/// using string keys in a hash map.
#[derive(Resource)]
pub struct ProjectileSpawners(pub HashMap<String, SystemId>);

pub fn spawn_fire_projectile(
    In(image): In<&Handle<Image>>,
    In(caster_pos): In<Vec2>,
    In(caster_aim_pos): In<Vec2>,
    mut commands: Commands,
    mut weapon_query: Query<(&mut ProjectileWeapon, &Parent)>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    SpellFactory::spawn_spell(
        &mut commands,
        Spell::Fireball,
        image,
        caster_pos,
        caster_aim_pos,
        todo!(),
    );
}

pub struct SpellFactory;

impl SpellFactory {
    pub fn spawn_spell(
        commands: &mut Commands,
        spell: Spell,
        caster_position: Vec2,
        caster_aim_position: Vec2,
        spell_image: &Handle<Image>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) {
        let spell_speed = 300.0;
        let angle = caster_position.angle_to(caster_aim_position);

        let mut transform = Transform {
            translation: caster_position.extend(0.0),
            ..default()
        };

        transform.rotate_z(angle);
        let direction = Vec3::from(transform.rotation.to_euler(EulerRot::XYZ)).truncate();
        let velocity = direction * spell_speed;

        match spell {
            Spell::Fireball => {
                commands
                    .spawn((
                        spell,
                        transform,
                        CollisionDamage::default(),
                        LinearVelocity(velocity),
                        EffectsList {
                            effects: vec![ApplyStatus {
                                status: StatusType::Burning(BurningStatus::default()),
                                duration: 2.0,
                            }],
                        },
                        Sprite::from_image(spell_image.clone()),
                    ))
                    .observe(on_damage_dealt_despawn);
            }
            Spell::Icebolt => {
                let animation_indices = AnimationIndices { first: 0, last: 4 };
                let texture = spell_image.clone();
                let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);

                commands
                    .spawn((
                        spell,
                        CollisionDamage { damage: 8.0 },
                        LinearVelocity(velocity),
                        EffectsList {
                            effects: vec![ApplyStatus {
                                status: StatusType::Frozen,
                                duration: 2.0,
                            }],
                        },
                        Sprite::from_atlas_image(
                            texture,
                            TextureAtlas {
                                layout: texture_atlas_layout,
                                index: animation_indices.first,
                            },
                        ),
                        transform,
                        animation_indices,
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    ))
                    .observe(on_damage_dealt_despawn);
            }
        }
    }
}
