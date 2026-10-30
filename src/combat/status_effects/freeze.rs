use bevy::color::palettes::tailwind::BLUE_400;
use bevy::prelude::*;
use bevy_lit::prelude::PointLight2d;

use super::status::{StatusApplied, StatusOf, StatusType, StatusVfxOf};
use crate::prelude::*;

#[derive(Component, Clone, Default)]
#[require(StatusType::Freeze)]
pub struct Frozen;

pub(super) fn apply_frozen(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Frozen>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);
        commands
            .entity(status_of.0)
            .with_child(grounded_ice_vfx(&sprites, status));
    }
}

fn grounded_ice_vfx(sprites: &SpriteAssets, status: Entity) -> impl Bundle {
    (
        Sprite::from_image(sprites.grounded_ice.clone()),
        StatusVfxOf(status),
        PointLight2d {
            color: Color::from(BLUE_400),
            intensity: 1.4,
            falloff: 5.0,
            outer_radius: 80.0,
            ..default()
        },
        Transform {
            translation: Vec3::new(
                0.0,
                CHARACTER_FEET_POS_OFFSET + 16.0,
                ZLayer::SpriteForeground.z(),
            ),
            scale: Vec3::new(1.4, 1.4, 1.0),
            ..default()
        },
    )
}
