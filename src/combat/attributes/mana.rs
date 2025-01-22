use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Mana {
    pub current_mana: f32,
    pub max_mana: f32,
}

impl Mana {
    pub fn new(max_mana: f32) -> Self {
        Self {
            current_mana: max_mana,
            max_mana,
        }
    }

    // optionally uses mana if it can afford it, otherwise returns false if it cost too much
    pub fn use_mana(&mut self, cost: f32) -> bool {
        if (self.current_mana - cost) >= 0.0 {
            self.current_mana = self.current_mana - cost;
            return true;
        }
        return false;
    }
}

/**
 * Attach it to projectiles, weapons, spells, etc... if they cost mana to use
 */
#[derive(Component, Clone)]
pub struct ManaCost(pub f32);
