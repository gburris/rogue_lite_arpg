use std::cmp::Ordering;

use bevy::prelude::*;

use super::status::{StatusKind, Statuses};

/// A stat a [`StatModifier`] changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stat {
    Speed,
    Damage,
}

/// Changes one stat by `change` while it lasts: `0.3` raises it 30%, `-0.5` halves it. It has no
/// visual of its own.
#[derive(Component, Clone, Copy, Debug)]
pub struct StatModifier {
    pub stat: Stat,
    pub change: f32,
}

impl StatModifier {
    /// Whether this modifier, with `remaining_secs` to run, replaces `existing` of the same kind:
    /// the bigger change wins and the longer remaining time breaks a tie, so a weak or short effect
    /// never cuts a strong one short.
    pub(super) fn outranks(
        self,
        remaining_secs: f32,
        existing: Self,
        existing_remaining_secs: f32,
    ) -> bool {
        match self.change.abs().total_cmp(&existing.change.abs()) {
            Ordering::Greater => true,
            Ordering::Less => false,
            Ordering::Equal => remaining_secs >= existing_remaining_secs,
        }
    }
}

/// A [`StatModifier`] effect changing `stat` by `change`. A raise and a lowering are separate
/// status kinds per stat, so one effect list can modify several stats and a lowering never replaces
/// a raise.
pub fn stat_modifier(stat: Stat, change: f32) -> impl Scene {
    assert!(
        change >= -1.0,
        "lowering a stat removes at most the whole stat, got {change}"
    );
    let kind = if change >= 0.0 {
        StatusKind::StatRaise { stat }
    } else {
        StatusKind::StatLower { stat }
    };
    bsn! {
        ~{StatModifier { stat, change }}
        ~{kind}
    }
}

/// The multiples of a character's stats its statuses leave: `1.0` unchanged, above `1.0` raised,
/// below lowered, and a `speed` of `0.0` stunned.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[component(immutable)]
pub struct StatMultipliers {
    pub speed: f32,
    pub damage: f32,
}

impl Default for StatMultipliers {
    fn default() -> Self {
        Self {
            speed: 1.0,
            damage: 1.0,
        }
    }
}

impl StatMultipliers {
    /// Whether its speed is lowered to nothing, as by a freeze.
    pub fn is_stunned(self) -> bool {
        self.speed <= 0.0
    }
}

/// Sets each character's [`StatMultipliers`] from its statuses: per stat, the strongest raise times
/// the strongest lowering, so repeated modifiers never compound and one expiring never clears
/// another.
pub(super) fn sync_stat_multipliers_from_statuses(
    mut commands: Commands,
    characters: Query<(Entity, Option<&Statuses>, &StatMultipliers)>,
    modifiers: Query<&StatModifier>,
) {
    for (character, statuses, current_multipliers) in &characters {
        let multipliers = StatMultipliers {
            speed: stat_multiplier(statuses, &modifiers, Stat::Speed),
            damage: stat_multiplier(statuses, &modifiers, Stat::Damage),
        };
        if multipliers != *current_multipliers {
            commands.entity(character).try_insert(multipliers);
        }
    }
}

/// The strongest raise `statuses` give `stat` times their strongest lowering.
fn stat_multiplier(
    statuses: Option<&Statuses>,
    modifiers: &Query<&StatModifier>,
    stat: Stat,
) -> f32 {
    let (strongest_raise, strongest_lower) = statuses
        .into_iter()
        .flat_map(Statuses::iter)
        .filter_map(|status| modifiers.get(status).ok())
        .filter(|modifier| modifier.stat == stat)
        .fold((0.0_f32, 0.0_f32), |(raise, lower), modifier| {
            (raise.max(modifier.change), lower.min(modifier.change))
        });
    (1.0 + strongest_raise) * (1.0 + strongest_lower)
}
