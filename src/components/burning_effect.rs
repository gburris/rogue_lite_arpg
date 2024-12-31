use bevy::prelude::*;

#[derive(Component)]
pub struct BurningEffect {
    pub damage_per_second: f32,
    pub duration: Timer,
    pub tick_timer: Timer,
}
