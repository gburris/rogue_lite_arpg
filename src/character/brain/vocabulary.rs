use super::engine::{self, BrainVocabulary};
use super::inputs::DecisionInputs;
use super::{Action, ConsiderationInput, MotionKey};

/// The motions, actions and inputs of character brains, for the scoring engine.
#[derive(Clone, Copy, Debug)]
pub struct CharacterVocabulary;

impl BrainVocabulary for CharacterVocabulary {
    type Motion = MotionKey;
    type Action = Action;
    type Input = ConsiderationInput;

    /// Idling never reports a finish. A sustained use doesn't either, which depends on the
    /// character's source; see [`Behavior::suits_commit_policy`].
    fn action_reports_finish(action: Action) -> bool {
        match action {
            Action::Idle => false,
            Action::Use(_) => true,
        }
    }
}

/// A behavior of a character brain.
pub type Behavior = engine::Behavior<CharacterVocabulary>;

impl Behavior {
    /// Whether the deciding character's sources for this behavior's motion and action end on
    /// their own in the way its commit policy needs: a committed behavior needs exactly one part
    /// that does, since nothing else would end it, and a continuously scored one needs a motion
    /// that runs until replaced.
    pub(super) fn suits_commit_policy(&self, inputs: &DecisionInputs) -> bool {
        let finite_motion = self
            .motion
            .is_some_and(|key| inputs.motion_ends_on_its_own(key));
        if !self.commit.is_committed() {
            return !finite_motion;
        }
        let finite_action = self
            .action
            .is_some_and(|action| inputs.action_ends_on_its_own(action));
        usize::from(finite_motion) + usize::from(finite_action) == 1
    }
}

/// A consideration of a character brain.
pub type Consideration = engine::Consideration<CharacterVocabulary>;
/// A character brain's static definition.
pub type BrainDefinition = engine::BrainDefinition<CharacterVocabulary>;
/// How a behavior of a character brain scored.
pub type BehaviorScoreExplanation = engine::BehaviorScoreExplanation<CharacterVocabulary>;
