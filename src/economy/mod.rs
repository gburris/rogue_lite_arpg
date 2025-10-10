mod gold;

use bevy::prelude::*;

use crate::labels::sets::InGameSet;

pub struct EconomyPlugin;

/// Shop and Gold Logic
impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            gold::handle_gold_collisions.in_set(InGameSet::Collision),
        )
        .add_observer(gold::on_gold_drop_event);
    }
}

pub use gold::Gold;
pub use gold::GoldDrop;

#[derive(Component, Clone, Default)]
pub struct Purse {
    pub amount: u32,
}

impl Purse {
    pub fn add(&mut self, amount: u32) {
        self.amount += amount;
    }

    pub fn remove(&mut self, amount: u32) -> Result<u32, String> {
        if self.amount >= amount {
            self.amount -= amount;
            Ok(self.amount)
        } else {
            Err("Not enough in purse!".to_string())
        }
    }
}
