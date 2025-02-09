use bevy::prelude::*;

#[derive(Component)]
pub struct Wallet {
    pub coins: u32,
}

impl Default for Wallet {
    fn default() -> Self {
        Self { coins: 0 }
    }
}

impl Wallet {
    pub fn add_currency(&mut self, currency: &Currency) {
        self.coins += currency.value;
    }
}
// Currency
// This is needed to tell Bevy to not treat gold as an item (into inventory)
// But rather a curreny (into wallet)
#[derive(Component)]
pub struct Currency {
    pub value: u32,
}

#[derive(Event)]
pub struct GoldDropEvent {
    pub drop_location: Transform,
    pub amount: u32,
}
