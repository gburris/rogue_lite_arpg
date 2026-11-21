use std::time::Duration;

use bevy::prelude::*;

use crate::interval::fraction_of_max;
use crate::prelude::*;

#[derive(EntityEvent)]
pub struct AttemptHeal {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(EntityEvent)]
pub struct Healed {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(Component, Clone)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn new(max_hp: f32) -> Self {
        Self { hp: max_hp, max_hp }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.hp = (self.hp - amount).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Current health as a `0.0..=1.0` fraction, or `None` when the health
    /// values cannot describe a valid fraction.
    pub fn fraction(&self) -> Option<f32> {
        fraction_of_max(self.hp, self.max_hp)
    }

    fn add_health(&mut self, amount: f32) -> f32 {
        let before = self.hp;
        self.hp += amount;
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
        self.hp - before
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}

pub(super) fn on_attempt_heal(
    attempt_heal: On<AttemptHeal>,
    mut commands: Commands,
    mut healed_query: Query<&mut Health>,
) {
    if let Ok(mut health) = healed_query.get_mut(attempt_heal.entity) {
        let actual_amount = health.add_health(attempt_heal.amount);
        // A heal at full health restores nothing, so it isn't reported.
        if actual_amount > 0.0 {
            commands.trigger(Healed {
                entity: attempt_heal.entity,
                amount: actual_amount,
            });
        }
    }
}

/// Seconds an entity with [`IFrames`] ignores damage after a hit.
const INVULNERABLE_SECONDS: f32 = 0.5;

/// Component to mark whether an entity has iframes when hit.
#[derive(Component)]
pub struct IFrames {
    // time to be invulnerable when hit
    pub invulnerable: bool,
    invulnerable_timer: Timer,
    flash_timer: Option<Timer>,
}

impl Default for IFrames {
    fn default() -> Self {
        Self {
            invulnerable: false,
            invulnerable_timer: Timer::from_seconds(INVULNERABLE_SECONDS, TimerMode::Once),
            flash_timer: Some(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        }
    }
}

impl IFrames {
    fn reset(&mut self) {
        self.invulnerable = false;
        self.invulnerable_timer.reset();

        if let Some(flash) = &mut self.flash_timer {
            flash.reset();
        }
    }
}

/// Counts down invulnerability and flashes the sprite until it ends.
pub(super) fn tick_invulnerability(time: Res<Time>, mut query: Query<(&mut IFrames, &mut Sprite)>) {
    for (mut iframes, mut sprite) in &mut query {
        if iframes.invulnerable {
            iframes.invulnerable_timer.tick(time.delta());

            if iframes.invulnerable_timer.is_finished() {
                iframes.reset();
                sprite.color.set_alpha(1.0);
            } else if let Some(flash) = &mut iframes.flash_timer {
                flash.tick(time.delta());

                if flash.just_finished() {
                    let current_alpha = sprite.color.alpha();
                    sprite.color.set_alpha(1.1 - current_alpha);
                }
            }
        }
    }
}

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
        self.current >= *cost
    }

    /// Current mana as a `0.0..=1.0` fraction, or `None` when the values can't form one.
    pub fn fraction(&self) -> Option<f32> {
        fraction_of_max(self.current, self.max)
    }

    /// Subtracts `cost`, flooring at zero.
    pub fn spend(&mut self, cost: ManaCost) {
        self.current = (self.current - *cost).max(0.0);
    }

    pub fn regenerate(&mut self, delta_time: f32) {
        self.current = (self.current + self.regen_rate * delta_time).min(self.max);
    }
}

/// Attach it to projectiles, weapons, spells, etc... if they cost mana to use.
#[derive(Component, Clone, Copy, Debug, Default, Deref)]
pub struct ManaCost(pub f32);

impl ManaCost {
    /// Whether `mana` covers this cost. A character without a mana pool can't pay any cost.
    pub fn is_affordable_by(self, mana: Option<&Mana>) -> bool {
        mana.is_some_and(|mana| mana.can_afford(self))
    }
}

/// Mana per second an action source drains from its holder while a use of it is in progress.
#[derive(Component, Clone, Copy, Debug, Default, Deref)]
pub struct ManaDrainRate(pub f32);

impl ManaDrainRate {
    /// The mana this rate drains over `delta_seconds`.
    pub fn cost_over(self, delta_seconds: f32) -> ManaCost {
        ManaCost(*self * delta_seconds)
    }
}

/// Regenerates all `Mana` in game based on time elapsed and the given mana instance's regeneration
/// rate.
pub(super) fn regenerate_mana(mut query: Query<&mut Mana>, time: Res<Time>) {
    let delta_time = time.delta_secs();
    for mut mana in &mut query {
        mana.regenerate(delta_time);
    }
}

/// Drains each in-progress action's [`ManaDrainRate`] from its holder, and stops the use once the
/// holder can't pay.
pub(super) fn drain_mana_for_actions_in_progress(
    mut commands: Commands,
    time: Res<Time>,
    draining_sources: Query<(Entity, &ManaDrainRate, &ActionInProgress)>,
    mut holder_query: Query<&mut Mana>,
) {
    for (source, &drain_rate, in_progress) in &draining_sources {
        let Ok(mut mana) = holder_query.get_mut(in_progress.holder()) else {
            continue;
        };
        let drain_amount = drain_rate.cost_over(time.delta_secs());
        if mana.can_afford(drain_amount) {
            mana.spend(drain_amount);
        } else {
            commands.trigger(StopAction { source });
        }
    }
}
