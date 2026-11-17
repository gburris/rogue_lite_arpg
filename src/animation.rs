use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, animate_sprites.in_set(VfxSystems::Animate));
}

/// A sprite showing the first frame of `image` as cut by `layout`.
pub fn atlas_sprite(image: Handle<Image>, layout: Handle<TextureAtlasLayout>) -> impl Scene {
    let sprite = Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 });
    bsn! {
        template(move |_| Ok(sprite.clone()))
    }
}

/// An effect that plays the `frame_count` frames of `image` once, `frame_seconds` apart, then
/// despawns.
pub fn one_shot_vfx(
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frame_count: u16,
    frame_seconds: f32,
) -> impl Scene {
    let last_frame = usize::from(frame_count.saturating_sub(1));
    let lifespan_seconds = f32::from(frame_count) * frame_seconds;
    bsn! {
        @atlas_sprite(image, layout)
        ~{AnimationIndices::OneShot(0..=last_frame)}
        AnimationTimer(Timer::from_seconds(frame_seconds, TimerMode::Repeating))
        Lifespan::new(lifespan_seconds)
    }
}

/// An effect that loops the `frame_count` frames of `image`, `frame_seconds` apart, until it is
/// despawned.
pub fn looping_vfx(
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frame_count: u16,
    frame_seconds: f32,
) -> impl Scene {
    let last_frame = usize::from(frame_count.saturating_sub(1));
    bsn! {
        @atlas_sprite(image, layout)
        ~{AnimationIndices::Cycle((0..=last_frame).cycle())}
        AnimationTimer(Timer::from_seconds(frame_seconds, TimerMode::Repeating))
    }
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
