use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Mana {
    pub current_mana: f32,
    pub max_mana: f32,
    pub regen_rate: f32,
}

impl Mana {
    pub fn new(max_mana: f32, regen_rate: f32) -> Self {
        Self {
            current_mana: max_mana,
            max_mana,
            regen_rate,
        }
    }

    pub fn has_enough_mana(&self, cost: &ManaCost) -> bool {
        self.current_mana >= cost.0
    }

    /// Current mana as a `0.0..=1.0` fraction, or `None` for an invalid maximum.
    pub fn fraction(&self) -> Option<f32> {
        if self.max_mana.is_finite() && self.max_mana > 0.0 {
            Some((self.current_mana / self.max_mana).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    pub fn use_mana(&mut self, cost: &ManaCost) {
        self.current_mana = (self.current_mana - cost.0).max(0.0);
    }

    pub fn regenerate(&mut self, delta_time: f32) {
        self.current_mana += self.regen_rate * delta_time;
        if self.current_mana > self.max_mana {
            self.current_mana = self.max_mana;
        }
    }
}

/// Attach it to projectiles, weapons, spells, etc... if they cost mana to use.
#[derive(Component, Clone)]
pub struct ManaCost(pub f32);

#[derive(Component, Clone)]
pub struct ManaDrainRate(pub f32);

/// Regenerates all `Mana` in game based on time elapsed and the given mana instance's regeneration
/// rate.
pub(crate) fn regenerate_mana(mut query: Query<&mut Mana>, time: Res<Time>) {
    let delta_time = time.delta_secs();
    for mut mana in &mut query {
        mana.regenerate(delta_time);
    }
}
