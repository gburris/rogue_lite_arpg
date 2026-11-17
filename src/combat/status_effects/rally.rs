use bevy::prelude::*;

use super::status::{StatusApplied, StatusKind, StatusOf, StatusVfxOf};
use crate::prelude::*;

/// Rallied by a war cry. Only the look: the effect list that rallies carries its
/// [`StatModifier`](super::stat_modifier::StatModifier)s.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(StatusKind::Rally)]
pub struct Rallied;

/// Marks a rallied character with rising arrows at its feet for as long as the status lasts.
pub(super) fn apply_rallied(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Rallied>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);
        let target = status_of.0;
        commands.spawn_scene(bsn! {
            @rallied_aura_vfx(&sprites, &sprite_layouts, status)
            ChildOf(target)
        });
    }
}

fn rallied_aura_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    status: Entity,
) -> impl Scene {
    bsn! {
        @looping_vfx(sprites.rallied_aura.clone(), sprite_layouts.rallied_aura.clone(), 12, 0.083)
        StatusVfxOf(status)
        // The sheet's feet sit 14 px below its centre.
        Transform::from_xyz(0.0, {CHARACTER_FEET_POS_OFFSET + 14.0}, {ZLayer::SpriteForeground.z()})
    }
}
