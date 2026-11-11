//! [`MotionActionTable`], one value per motion and per action, and the keys that index it.

use super::action::{Ability, Action};
use super::motion::Motion;

/// A type with its own entry in a [`MotionActionTable`].
pub(super) trait MotionActionKey: Copy {
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
pub(super) struct MotionActionTable<T>([T; Motion::COUNT + Action::COUNT]);

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
