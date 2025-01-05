use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Default for Health {
    fn default() -> Self {
        Health {
            hp: 100.0,
            max_hp: 100.0,
        }
    }
}
