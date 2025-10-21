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
        };
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
    for (mut indices, mut timer, mut sprite, state, direction) in query.iter_mut() {
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
            AnimationIndices::Cycle(cycle) => cycle.clone().next().unwrap_or_default(),
            AnimationIndices::OneShot(range_inclusive) => *range_inclusive.start(),
        }
    }
}

impl Iterator for AnimationIndices {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AnimationIndices::Cycle(cycle) => cycle.next(),
            AnimationIndices::OneShot(range_inclusive) => range_inclusive.next(),
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

impl Default for DefaultAnimationConfig {
    fn default() -> Self {
        let mut animations = HashMap::default();

        // Define all animations
        animations.insert(
            (ActionState::Idle, FacingDirection::Down),
            AnimationData {
                row: 14,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (ActionState::Idle, FacingDirection::Up),
            AnimationData {
                row: 12,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (ActionState::Idle, FacingDirection::Left),
            AnimationData {
                row: 13,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );
        animations.insert(
            (ActionState::Idle, FacingDirection::Right),
            AnimationData {
                row: 15,
                frame_count: 3,
                frame_duration: 0.5,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Down),
            AnimationData {
                row: 10,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Up),
            AnimationData {
                row: 8,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Left),
            AnimationData {
                row: 9,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Movement, FacingDirection::Right),
            AnimationData {
                row: 11,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );
        //Literally less code to repeat this 4x than solve it in a proper way
        //All four FacingDirections map to defeated down row / animation
        animations.insert(
            (ActionState::Defeated, FacingDirection::Down),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (ActionState::Defeated, FacingDirection::Left),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );
        animations.insert(
            (ActionState::Defeated, FacingDirection::Right),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (ActionState::Defeated, FacingDirection::Up),
            AnimationData {
                row: 20,
                frame_count: 5,
                frame_duration: 0.4,
            },
        );

        animations.insert(
            (ActionState::Attacking, FacingDirection::Up),
            AnimationData {
                row: 16,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Attacking, FacingDirection::Down),
            AnimationData {
                row: 18,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );
        animations.insert(
            (ActionState::Attacking, FacingDirection::Left),
            AnimationData {
                row: 17,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

        animations.insert(
            (ActionState::Attacking, FacingDirection::Right),
            AnimationData {
                row: 19,
                frame_count: 9,
                frame_duration: 0.1,
            },
        );

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
        self.animations
            .get(&(state, direction))
            .ok_or(format!("Missing animation data for {state:?} {direction:?}").into())
    }

    pub fn indices(
        &self,
        state: ActionState,
        direction: FacingDirection,
    ) -> Result<AnimationIndices> {
        let animation = self.animation(state, direction)?;
        let first = animation.row * self.columns;
        let last = first + animation.frame_count - 1;
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
