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

    // optionally uses mana if it can afford it, otherwise returns false if it cost too much
    pub fn use_mana(&mut self, cost: f32) -> bool {
        if (self.current_mana - cost) >= 0.0 {
            self.current_mana -= cost;
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

/**
 * Attach it to projectiles, weapons, spells, etc... if they cost mana to use
 */
#[derive(Component, Clone)]
pub struct ManaCost(pub f32);
