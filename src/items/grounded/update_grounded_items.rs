// System to update visual effects for grounded items
use ::bevy::prelude::*;

use crate::items::{Grounded, GroundedItemEffect};

pub fn update_grounded_items(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut GroundedItemEffect), With<Grounded>>,
) {
    for (entity, mut transform, mut effect) in query.iter_mut() {
        // Update rotation
        effect.rotation += time.delta_secs() * 0.5; // Adjust speed as needed
        transform.rotation = Quat::from_rotation_z(effect.rotation);

        // Update glow effect
        effect.glow_offset += time.delta_secs() * 2.0;

        // Create or update glow sprites
        let glow_positions = [
            Vec3::new(10.0, 0.0, -0.1),  // Right
            Vec3::new(-10.0, 0.0, -0.1), // Left
            Vec3::new(0.0, 10.0, -0.1),  // Top
            Vec3::new(0.0, -10.0, -0.1), // Bottom
        ];

        // You might want to handle this more efficiently in practice
        commands.entity(entity).with_children(|parent| {
            for pos in glow_positions.iter() {
                let glow_offset = (effect.glow_offset + pos.x + pos.y).sin() * 2.0;
                let glow_pos = *pos + Vec3::new(0.0, glow_offset, 0.0);

                // parent.spawn((SpriteBundle {
                //     sprite: Sprite {
                //         color: Color::rgba(1.0, 1.0, 0.0, 0.3), // Yellow glow
                //         custom_size: Some(Vec2::new(4.0, 4.0)),
                //         ..default()
                //     },
                //     transform: Transform::from_translation(glow_pos),
                //     ..default()
                // },));
            }
        });
    }
}
