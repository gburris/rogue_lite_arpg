mod vfx;

use bevy::prelude::*;

use crate::labels::sets::InGameSystems;

pub use vfx::heal_vfx;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprite.in_set(InGameSystems::Vfx));
    }
}

#[derive(Clone, Debug, Component)]
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
            AnimationIndices::Cycle(cycle) => cycle.clone().next().unwrap_or_default(),
            AnimationIndices::OneShot(range_inclusive) => *range_inclusive.start(),
        }
    }
}

impl Default for AnimationIndices {
    fn default() -> Self {
        Self::OneShot(0..=0)
    }
}

#[derive(Component, Deref, DerefMut, Default, Clone)]
#[require(AnimationIndices)]
pub struct AnimationTimer(pub Timer);

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub row: usize,          // Row in the sprite sheet
    pub frame_count: usize,  // Number of frames in animation
    pub frame_duration: f32, // Duration of each frame
}

fn animate_sprite(
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
