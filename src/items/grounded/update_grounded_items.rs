use std::f32::consts::PI;

use ::bevy::prelude::*;

use crate::items::{Grounded, Item};

pub fn update_grounded_items(
    mut query: Query<(Entity, &mut Item, &mut Transform, &mut Sprite), With<Grounded>>,
    time: Res<Time>,
) {
    for (_entity, mut item, mut transform, mut sprite) in query.iter_mut() {
        item.drop_rotation_timer += time.delta_secs();
        let rotation_angle = (item.drop_rotation_timer / 6.0) * 2.0 * PI;
        transform.rotation = Quat::from_rotation_z(rotation_angle);
        item.drop_glow_effect += time.delta_secs() * 2.0;
        let glow_intensity = (item.drop_glow_effect.sin() * 0.1 + 0.7) as f32;
        let base_color = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            base_color.red * glow_intensity + 0.3,
            base_color.green * glow_intensity + 0.3,
            base_color.blue * glow_intensity + 0.3,
            base_color.alpha,
        );
    }
}
