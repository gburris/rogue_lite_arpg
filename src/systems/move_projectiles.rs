use crate::components::Projectile;
use bevy::prelude::*;

pub fn move_projectiles(
    mut query: Query<(&mut Transform, &Projectile)>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in &mut query {
        let direction = transform.local_x();
        transform.translation += direction * projectile.speed * time.delta_secs();
    }
}

