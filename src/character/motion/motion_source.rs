use bevy::prelude::*;

/// Names a motion source, so a brain can pick which of a character's motions to run. The source's
/// tuning component decides how the motion steers and requires its key; a
/// [`Dash`](super::dash::Dash) serves several keys.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKey {
    Idle,
    Wander,
    ReturnToHome,
    Chase,
    Flee,
    KeepDistance,
    Orbit,
    Kite,
    Slide,
    Sidestep,
    StepIn,
    Swoop,
}

impl MotionKey {
    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Wander => "Wander",
            Self::ReturnToHome => "ReturnToHome",
            Self::Chase => "Chase",
            Self::Flee => "Flee",
            Self::KeepDistance => "KeepDistance",
            Self::Orbit => "Orbit",
            Self::Kite => "Kite",
            Self::Slide => "Slide",
            Self::Sidestep => "Sidestep",
            Self::StepIn => "StepIn",
            Self::Swoop => "Swoop",
        }
    }
}

/// Links a motion source to the character that moves with it. A source is an entity with a
/// [`MotionKey`] and that motion's tuning, such as a [`Dash`](super::dash::Dash).
#[derive(Component, Clone)]
#[relationship(relationship_target = MotionSources)]
pub struct MotionSourceOf {
    pub character: Entity,
}

/// A character's motion sources, one for each motion it can run.
#[derive(Component, Clone)]
#[relationship_target(relationship = MotionSourceOf, linked_spawn)]
pub struct MotionSources(Vec<Entity>);

/// Checks in debug builds that no other source of the new motion source's character has its key,
/// since motions are found by key.
pub(super) fn on_motion_source_added(
    added: On<Add<MotionSourceOf>>,
    sources: Query<(&MotionSourceOf, &MotionKey)>,
    characters: Query<&MotionSources>,
) {
    let Ok((&MotionSourceOf { character }, &key)) = sources.get(added.entity) else {
        return;
    };
    let Ok(character_sources) = characters.get(character) else {
        return;
    };
    // The new source may not be in `MotionSources` yet, so only the others are counted.
    let other_same_key_count = character_sources
        .iter()
        .filter(|&source| source != added.entity)
        .filter(|&source| sources.get(source).is_ok_and(|(_, &other)| other == key))
        .count();
    debug_assert!(
        other_same_key_count == 0,
        "character {character:?} has {other_same_key_count} more motion sources keyed {key:?}; \
         only the first would ever run"
    );
}

/// Marks a motion source whose motion ends on its own, rather than running until replaced. Its end
/// starts the source's [`Cooldown`](crate::prelude::Cooldown) and reports
/// [`MotionFinished`](super::MotionFinished).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct FiniteMotion;
