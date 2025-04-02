
use bevy::prelude::*;

use super::{AnimationIndices, AnimationTimer};

pub fn animate_sprite(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut Sprite,
    )>,
) {
    for (entity, mut indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };
        if !timer.just_finished() {
            continue;
        }
        match &mut *indices {
            AnimationIndices::None(i) => {}
            AnimationIndices::Cycle(i) => {
                if let Some(index) = i.next() {
                    atlas.index = index
                };
            }
            AnimationIndices::OneShot(i) => {
                match i.next() {
                    Some(index) => atlas.index = index,
                    None => {
                        commands.entity(entity).remove::<AnimationTimer>();
                    }
                };
            }
        };
    }
}
