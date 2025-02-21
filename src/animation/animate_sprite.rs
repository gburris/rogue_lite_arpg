use bevy::prelude::*;

use super::{AnimationIndices, AnimationTimer};

pub fn animate_sprite(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (entity, indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if indices.is_one_shot && atlas.index == indices.last {
                    commands.entity(entity).remove::<AnimationTimer>();
                    return;
                }

                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}
