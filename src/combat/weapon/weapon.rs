use bevy::prelude::*;

#[derive(Component)]
pub struct Weapon {
    pub should_attack: bool, // swing a sword, shoot a weapon, etc...
    pub attack_rate: Timer,
}

impl Default for Weapon {
    fn default() -> Self {
        Weapon {
            should_attack: false,
            attack_rate: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}

pub fn tick_weapon_attack_rate(mut weapon_query: Query<&mut Weapon>, time: Res<Time>) {
    for mut weapon in weapon_query.iter_mut() {
        weapon.attack_rate.tick(time.delta());
    }
}
