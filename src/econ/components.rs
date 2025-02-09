use bevy::prelude::*;

#[derive(Component)]
pub struct Wallet {
    pub coins: f32,
}

impl Default for Wallet {
    fn default() -> Self {
        Self { coins: 0.0 }
    }
}

// Components
#[derive(Component)]
pub struct Magnet;

#[derive(Event)]
pub struct GoldDropEvent {
    pub drop_location: Transform,
    pub amount: u32,
}
