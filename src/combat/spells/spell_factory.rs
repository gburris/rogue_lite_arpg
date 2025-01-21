use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::projectile::{
    components::ProjectileBundle, on_damage_dealt::on_damage_dealt_despawn,
};

pub struct SpellFactory;

impl SpellFactory {
    pub fn spawn_spell(
        commands: &mut Commands,
        caster_position: Vec2,
        caster_aim_position: Vec2,
        projectile_bundle: &ProjectileBundle,
    ) {
        let spell_speed = 300.0;

        let mut transform = Transform {
            translation: caster_position.extend(0.0),
            ..default()
        };

        let direction = caster_aim_position - caster_position;
        let angle = caster_aim_position.angle_to(direction);

        transform.rotate_z(angle);

        let velocity = direction.normalize() * spell_speed;

        warn!(
            "Spawning projectile w/ direction: {}, angle: {}",
            direction, angle
        );

        commands
            .spawn((
                projectile_bundle.clone(),
                Visibility::Visible,
                transform,
                LinearVelocity(velocity),
            ))
            .observe(on_damage_dealt_despawn);

        // Spell::Icebolt => {
        //     let animation_indices = AnimationIndices { first: 0, last: 4 };
        //     let texture = spell_image.clone();
        //     let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 5, 1, None, None);
        //     let texture_atlas_layout = texture_atlas_layouts.add(layout);

        //     commands
        //         .spawn((
        //             spell,
        //             CollisionDamage { damage: 8.0 },
        //             LinearVelocity(velocity),
        //             EffectsList {
        //                 effects: vec![ApplyStatus {
        //                     status: StatusType::Frozen,
        //                     duration: 2.0,
        //                 }],
        //             },
        //             Sprite::from_atlas_image(
        //                 texture,
        //                 TextureAtlas {
        //                     layout: texture_atlas_layout,
        //                     index: animation_indices.first,
        //                 },
        //             ),
        //             transform,
        //             animation_indices,
        //             AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        //         ))
        // }
    }
}
