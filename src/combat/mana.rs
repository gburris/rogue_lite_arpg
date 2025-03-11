use bevy::prelude::*;

#[derive(Component, Clone)]
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

    /// Optionally uses mana if it can afford it, otherwise returns false if it cost too much
    pub fn attempt_use_mana(&mut self, cost: &ManaCost) -> bool {
        if self.current_mana >= cost.0 {
            self.current_mana -= cost.0;
            return true;
        }
        return false;
    }

    pub fn regenerate(&mut self, delta_time: f32) {
        self.current_mana += self.regen_rate * delta_time;
        if self.current_mana > self.max_mana {
            self.current_mana = self.max_mana;
        }
    }
}

/// Attach it to projectiles, weapons, spells, etc... if they cost mana to use
#[derive(Component, Clone)]
pub struct ManaCost(pub f32);

/// Regenerates all `Mana` in game based on time elapsed and the given mana instance's regeneration rate
pub fn regenerate_mana(mut query: Query<&mut Mana>, time: Res<Time>) {
    let delta_time = time.delta_secs();
    for mut mana in query.iter_mut() {
        mana.regenerate(delta_time);
    }
}
