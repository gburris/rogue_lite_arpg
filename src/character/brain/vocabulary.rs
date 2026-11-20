use super::engine::{self, BrainVocabulary};
use super::{Action, ConsiderationInput, Motion};

/// The motions, actions and inputs of character brains, for the scoring engine.
#[derive(Clone, Copy, Debug)]
pub struct CharacterVocabulary;

impl BrainVocabulary for CharacterVocabulary {
    type Motion = Motion;
    type Action = Action;
    type Input = ConsiderationInput;

    /// Standing still never reports a finish. Whether a running motion does depends on the
    /// character's source, which decisions check when a behavior would start; see
    /// [`MotionStatus::ends_on_its_own`](crate::character::motion::MotionStatus::ends_on_its_own).
    fn motion_reports_finish(motion: Motion) -> bool {
        match motion {
            Motion::Idle => false,
            Motion::Run(_) => true,
        }
    }

    /// Idling never reports a finish. A sustained use doesn't either, which decisions check when a
    /// behavior would start; see
    /// [`ActionUseStyle::ends_on_its_own`](crate::prelude::ActionUseStyle::ends_on_its_own).
    fn action_reports_finish(action: Action) -> bool {
        match action {
            Action::Idle => false,
            Action::Use(_) => true,
        }
    }
}

/// A behavior of a character brain.
pub type Behavior = engine::Behavior<CharacterVocabulary>;
/// A consideration of a character brain.
pub type Consideration = engine::Consideration<CharacterVocabulary>;
/// A character brain's static definition.
pub type BrainDefinition = engine::BrainDefinition<CharacterVocabulary>;
/// How a behavior of a character brain scored.
pub type BehaviorScoreExplanation = engine::BehaviorScoreExplanation<CharacterVocabulary>;
