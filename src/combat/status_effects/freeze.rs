use bevy::color::palettes::tailwind::BLUE_400;
use bevy::prelude::*;

use super::status::{StatusApplied, StatusKind, StatusOf, StatusVfxOf};
use crate::prelude::*;

#[derive(Component, Clone, Default)]
#[require(StatusKind::Freeze)]
pub struct Frozen;

pub(super) fn apply_frozen(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Frozen>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);
        let target = status_of.0;
        commands.spawn_scene(bsn! {
            @grounded_ice_vfx(&sprites, status)
            ChildOf(target)
        });
    }
}

/// A sheet of ice under the frozen character's feet.
fn grounded_ice_vfx(sprites: &SpriteAssets, status: Entity) -> impl Scene {
    let image = sprites.grounded_ice.clone();
    bsn! {
        ~Sprite::from_image(image)
        ~{StatusVfxOf(status)}
        @point_light(Color::from(BLUE_400), 1.4, 80.0, 5.0)
        Transform {
            translation: Vec3::new(0.0, {CHARACTER_FEET_POS_OFFSET + 16.0}, {ZLayer::SpriteForeground.z()}),
            scale: Vec3::new(1.4, 1.4, 1.0),
        }
    }
}
