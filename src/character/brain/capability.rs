//! What an entity can do: the per-motion-and-action table and the profile each
//! capability component reports.

use super::action::{Ability, Action};
use super::motion::Motion;

/// A type with its own entry in a [`MotionActionTable`].
pub trait MotionActionKey: Copy {
    fn table_index(self) -> usize;
}

/// Motions come first in a [`MotionActionTable`].
impl MotionActionKey for Motion {
    fn table_index(self) -> usize {
        self.index()
    }
}

/// Actions follow the motions in a [`MotionActionTable`].
impl MotionActionKey for Action {
    fn table_index(self) -> usize {
        Motion::COUNT + self.index()
    }
}

impl MotionActionKey for Ability {
    fn table_index(self) -> usize {
        Action::Ability(self).table_index()
    }
}

/// One value per motion and per action, indexed by any [`MotionActionKey`].
#[derive(Clone, Copy, Debug)]
pub struct MotionActionTable<T>([T; Motion::COUNT + Action::COUNT]);

impl<T: Default> Default for MotionActionTable<T> {
    fn default() -> Self {
        Self(std::array::from_fn(|_| T::default()))
    }
}

impl<T, K: MotionActionKey> std::ops::Index<K> for MotionActionTable<T> {
    type Output = T;

    fn index(&self, key: K) -> &T {
        &self.0[key.table_index()]
    }
}

impl<T, K: MotionActionKey> std::ops::IndexMut<K> for MotionActionTable<T> {
    fn index_mut(&mut self, key: K) -> &mut T {
        &mut self.0[key.table_index()]
    }
}

/// What the decision needs to know about one capability, read from the component
/// that carries it out (see `DecisionInputs::motion_profile` and `action_profile`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CapabilityProfile {
    pub reach: Option<f32>,
    /// Probability that a winning finite motion actually starts.
    pub chance: f32,
    /// Seconds to wait after a finish or a missed start roll.
    pub cooldown: f32,
}

impl Default for CapabilityProfile {
    fn default() -> Self {
        Self {
            reach: None,
            chance: 1.0,
            cooldown: 0.0,
        }
    }
}
