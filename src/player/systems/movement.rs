use crate::components::Speed;
use crate::player::Player;
use crate::player::PlayerMovementEvent;
use crate::resources::MapBounds;
use crate::resources::PlayerSize;
use bevy::prelude::*;

// System to handle player movement based on movement events
pub fn player_movement(
    mut query: Query<(&mut Player, &mut Transform, &Speed)>,
    playersize: Res<PlayerSize>, // Player size to adjust clamping
    mapbounds: Res<MapBounds>,   // Map bounds for clamping
    mut event_reader: EventReader<PlayerMovementEvent>, // Listen for movement events
) {
    for event in event_reader.read() {
        // Process the movement event for each player
        for (_player, mut transform, speed) in query.iter_mut() {
            // Apply the movement direction from the event to the player's position
            transform.translation.x += event.direction.x * speed.velocity;
            transform.translation.y += event.direction.y * speed.velocity;

            // Clamp the player position within the map bounds
            let clamp_x = transform.translation.x.clamp(
                mapbounds.min_x + playersize.x / 2.0,
                mapbounds.max_x - playersize.x / 2.0,
            );
            let clamp_y = transform.translation.y.clamp(
                mapbounds.min_y + playersize.y / 2.0,
                mapbounds.max_y - playersize.y / 2.0,
            );

            // Update the transform to reflect the clamped position
            transform.translation = Vec3::new(clamp_x, clamp_y, 1.0);
        }
    }
}
