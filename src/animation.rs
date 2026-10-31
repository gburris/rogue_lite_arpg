use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, animate_sprites.in_set(VfxSystems::Animate));
}

fn animate_sprites(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut Sprite,
    )>,
) -> Result {
    for (entity, mut indices, mut timer, mut sprite) in &mut query {
        if !timer.tick(time.delta()).just_finished() {
            continue;
        }
        let atlas = sprite
            .texture_atlas
            .as_mut()
            .ok_or("Tried to animate a sprite without a texture atlas")?;
        let next = indices.next();
        match next {
            Some(index) => atlas.index = index,
            None => {
                commands.entity(entity).remove::<AnimationTimer>();
            }
        }
    }
    Ok(())
}

#[derive(Component, Clone, Debug)]
pub enum AnimationIndices {
    Cycle(std::iter::Cycle<std::ops::RangeInclusive<usize>>),
    OneShot(std::ops::RangeInclusive<usize>),
}
impl AnimationIndices {
    pub fn start(&self) -> usize {
        match self {
            // NOTE: this is not perfect, there's not easy way to access the original iterator
            // start which is what I would want.
            // TODO: Create helper functions to instantiate AnimationIndices types, that way it's
            // easier to include metadata
            Self::Cycle(cycle) => cycle.clone().next().unwrap_or_default(),
            Self::OneShot(range_inclusive) => *range_inclusive.start(),
        }
    }
}

impl Iterator for AnimationIndices {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Cycle(cycle) => cycle.next(),
            Self::OneShot(range_inclusive) => range_inclusive.next(),
        }
    }
}

impl Default for AnimationIndices {
    fn default() -> Self {
        Self::OneShot(0..=0)
    }
}

#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
#[require(AnimationIndices)]
pub struct AnimationTimer(pub Timer);

#[derive(Clone, Copy, Debug)]
pub struct AnimationData {
    pub row: usize,          // Row in the sprite sheet
    pub frame_count: usize,  // Number of frames in animation
    pub frame_duration: f32, // Duration of each frame
}
