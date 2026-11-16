use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::prelude::*;

/// Finds a character's allies: the other living characters on its side, gathered on each call.
#[derive(SystemParam)]
pub(super) struct Allies<'w, 's> {
    characters: Query<
        'w,
        's,
        (
            Entity,
            &'static Transform,
            &'static CombatSide,
            &'static Health,
        ),
    >,
}

/// An ally found by [`Allies`], and its distance from the character that asked.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ally {
    pub entity: Entity,
    pub distance: f32,
}

impl Allies<'_, '_> {
    /// The closest ally of `entity`, if any is alive.
    pub(super) fn closest(&self, entity: Entity) -> Option<Ally> {
        self.allies_of(entity)
            .min_by(|ally, other| ally.distance.total_cmp(&other.distance))
    }

    /// The allies of `entity` at most `reach` away, in no particular order.
    pub(super) fn within(&self, entity: Entity, reach: f32) -> impl Iterator<Item = Ally> {
        self.allies_of(entity)
            .filter(move |ally| ally.distance <= reach)
    }

    fn allies_of(&self, entity: Entity) -> impl Iterator<Item = Ally> {
        let caller = self.characters.get(entity).ok();
        caller
            .into_iter()
            .flat_map(move |(_, caller_transform, caller_side, _)| {
                let position = caller_transform.translation.xy();
                self.characters
                    .iter()
                    .filter(move |(ally, _, side, health)| {
                        *ally != entity && side == &caller_side && !health.is_dead()
                    })
                    .map(move |(ally, transform, ..)| Ally {
                        entity: ally,
                        distance: transform.translation.xy().distance(position),
                    })
            })
    }
}
