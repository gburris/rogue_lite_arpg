use bevy::prelude::*;

#[derive(Component, Clone, Copy, Default)]
pub struct Purse {
    pub amount: u32,
}

impl Purse {
    pub fn add(&mut self, amount: u32) {
        self.amount += amount;
    }
}
