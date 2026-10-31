use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
}

impl Mana {
    pub fn new(max: f32, regen_rate: f32) -> Self {
        Self {
            current: max,
            max,
            regen_rate,
        }
    }

    /// Whether the current mana covers `cost`.
    pub fn can_afford(&self, cost: ManaCost) -> bool {
        self.current >= cost.0
    }

    /// Current mana as a `0.0..=1.0` fraction, or `None` when the values can't form one.
    pub fn fraction(&self) -> Option<f32> {
        if self.current.is_finite() && self.max.is_finite() && self.max > 0.0 {
            Some((self.current / self.max).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    /// Subtracts `cost`, flooring at zero.
    pub fn spend(&mut self, cost: ManaCost) {
        self.current = (self.current - cost.0).max(0.0);
    }

    pub fn regenerate(&mut self, delta_time: f32) {
        self.current = (self.current + self.regen_rate * delta_time).min(self.max);
    }
}

/// Attach it to projectiles, weapons, spells, etc... if they cost mana to use.
#[derive(Component, Clone, Copy, Debug)]
pub struct ManaCost(pub f32);

#[derive(Component, Clone)]
pub struct ManaDrainRate(pub f32);

/// Regenerates all `Mana` in game based on time elapsed and the given mana instance's regeneration
/// rate.
pub(super) fn regenerate_mana(mut query: Query<&mut Mana>, time: Res<Time>) {
    let delta_time = time.delta_secs();
    for mut mana in &mut query {
        mana.regenerate(delta_time);
    }
}

/// Drains each in-use item's [`ManaDrainRate`] from its holder, and stops the use once the holder
/// can't pay.
pub(super) fn drain_mana_for_items_in_use(
    mut commands: Commands,
    time: Res<Time>,
    draining_items: Query<(Entity, &ManaDrainRate, &ItemOf), With<EquipmentInUse>>,
    mut holder_query: Query<&mut Mana>,
) {
    for (item, drain_rate, item_of) in &draining_items {
        let Ok(mut mana) = holder_query.get_mut(item_of.holder) else {
            continue;
        };
        let drain_amount = ManaCost(drain_rate.0 * time.delta_secs());
        if mana.can_afford(drain_amount) {
            mana.spend(drain_amount);
        } else {
            commands.trigger(StopUsingEquipment {
                item,
                holder: item_of.holder,
            });
        }
    }
}
