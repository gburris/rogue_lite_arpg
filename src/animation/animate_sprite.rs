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
        if !timer.tick(time.delta()).just_finished() {
            continue;
        }
        let atlas = sprite
            .texture_atlas
            .as_mut()
            .expect("Tried to animate a sprite without a texture atlas");
        let next = match &mut *indices {
            AnimationIndices::Cycle(i) => i.next(),
            AnimationIndices::OneShot(i) => i.next(),
        };
        match next {
            Some(index) => atlas.index = index,
            None => {
                commands.entity(entity).remove::<AnimationTimer>();
            }
        };
    }
}
