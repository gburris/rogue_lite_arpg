use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn new(max_hp: f32) -> Self {
        Self { hp: max_hp, max_hp }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.hp -= amount;
        if self.hp < 0.0 {
            self.hp = 0.0;
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Health {
            hp: 100.0,
            max_hp: 100.0,
        }
    }
}
