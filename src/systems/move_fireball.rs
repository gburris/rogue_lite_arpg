use crate::components::Fireball;
use bevy::prelude::*;

pub fn move_fireball(mut query: Query<(&mut Transform, &Fireball)>, time: Res<Time>) {
    for (mut transform, fireball) in &mut query {
        let direction: Dir3 = transform.local_x();
        transform.translation += direction * fireball.speed * time.delta_secs();
    }
}
