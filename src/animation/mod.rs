use bevy::{platform::collections::HashMap, prelude::*};
mod spells;

pub use spells::heal_vfx;

use crate::{
    labels::sets::InGameSystems,
    prelude::{ActionState, FacingDirection},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            animate_sprite_system,
            update_animation_system, //Change animation if components change that dictace updating it
        )
            .chain()
            .in_set(InGameSystems::Vfx),
    )
    .insert_resource(DefaultAnimationConfig::default());
}

pub fn animate_sprite_system(
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

pub fn update_animation_system(
    animation_config: Res<DefaultAnimationConfig>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &mut AnimationTimer,
            &mut Sprite,
            &ActionState,
            &FacingDirection,
        ),
        Or<(Changed<ActionState>, Changed<FacingDirection>)>,
    >,
) -> Result {
    for (mut indices, mut timer, mut sprite, state, direction) in &mut query {
        *indices = animation_config.indices(*state, *direction)?;
        *timer = AnimationTimer(animation_config.timer(*state, *direction)?);
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = indices.start();
        }
    }
    Ok(())
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

#[derive(Component, Deref, DerefMut, Default, Clone, Debug)]
#[require(AnimationIndices)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource)]
pub struct DefaultAnimationConfig {
    pub sprite_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animations: HashMap<(ActionState, FacingDirection), AnimationData>,
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub row: usize,          // Row in the sprite sheet
    pub frame_count: usize,  // Number of frames in animation
    pub frame_duration: f32, // Duration of each frame
}
impl From<(usize, usize, f32)> for AnimationData {
    fn from((row, frame_count, frame_duration): (usize, usize, f32)) -> Self {
        Self {
            row,
            frame_count,
            frame_duration,
        }
    }
}

impl Default for DefaultAnimationConfig {
    fn default() -> Self {
        // Define all animations
        let data = [
            (ActionState::Idle, FacingDirection::Down, (14, 3, 0.5)),
            (ActionState::Idle, FacingDirection::Up, (12, 3, 0.5)),
            (ActionState::Idle, FacingDirection::Left, (13, 3, 0.5)),
            (ActionState::Idle, FacingDirection::Right, (15, 3, 0.5)),
            (ActionState::Movement, FacingDirection::Down, (10, 9, 0.1)),
            (ActionState::Movement, FacingDirection::Up, (8, 9, 0.1)),
            (ActionState::Movement, FacingDirection::Left, (9, 9, 0.1)),
            (ActionState::Movement, FacingDirection::Right, (11, 9, 0.1)),
            //Literally less code to repeat this 4x than solve it in a proper way
            //All four FacingDirections map to defeated down row / animation
            (ActionState::Defeated, FacingDirection::Down, (20, 5, 0.4)),
            (ActionState::Defeated, FacingDirection::Left, (20, 5, 0.4)),
            (ActionState::Defeated, FacingDirection::Right, (20, 5, 0.4)),
            (ActionState::Defeated, FacingDirection::Up, (20, 5, 0.4)),
            (ActionState::Attacking, FacingDirection::Up, (16, 9, 0.1)),
            (ActionState::Attacking, FacingDirection::Down, (18, 9, 0.1)),
            (ActionState::Attacking, FacingDirection::Left, (17, 9, 0.1)),
            (ActionState::Attacking, FacingDirection::Right, (19, 9, 0.1)),
        ];
        let animations = data
            .into_iter()
            .map(|(state, dir, data)| ((state, dir), AnimationData::from(data)))
            .collect::<HashMap<_, _>>();

        // Add more animations as needed...

        Self {
            sprite_size: UVec2::new(64, 64),
            columns: 13,
            rows: 21,
            animations,
        }
    }
}

impl DefaultAnimationConfig {
    pub fn animation(
        &self,
        state: ActionState,
        direction: FacingDirection,
    ) -> Result<&AnimationData> {
        Ok(self
            .animations
            .get(&(state, direction))
            .ok_or_else(|| format!("Missing animation data for {state:?} {direction:?}"))?)
    }

    pub fn indices(
        &self,
        state: ActionState,
        direction: FacingDirection,
    ) -> Result<AnimationIndices> {
        let AnimationData {
            row, frame_count, ..
        } = self.animation(state, direction)?;
        let first = row * self.columns;
        let last = first + frame_count - 1;
        Ok(AnimationIndices::Cycle((first..=last).cycle()))
    }

    pub fn timer(&self, state: ActionState, direction: FacingDirection) -> Result<Timer> {
        let animation = self.animation(state, direction)?;
        Ok(Timer::from_seconds(
            animation.frame_duration,
            TimerMode::Repeating,
        ))
    }
}
