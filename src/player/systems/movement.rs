use bevy::prelude::*;

use crate::{
    components::Speed,
    map::resources::MapBounds,
    player::{Player, PlayerMovementEvent, ResetPlayerPosition},
    resources::PlayerSize,
};

// System to handle player movement based on movement events
pub fn player_movement(
    mut query: Query<(&mut Player, &mut Transform, &Speed)>,
    playersize: Res<PlayerSize>, // Player size to adjust clamping
    map_bounds: Res<MapBounds>,  // Map bounds for clamping
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
                map_bounds.min_x + playersize.x / 2.0,
                map_bounds.max_x - playersize.x / 2.0,
            );
            let clamp_y = transform.translation.y.clamp(
                map_bounds.min_y + playersize.y / 2.0,
                map_bounds.max_y - playersize.y / 2.0,
            );

            // Update the transform to reflect the clamped position
            transform.translation = Vec3::new(clamp_x, clamp_y, 1.0);
        }
    }
}

pub fn reset_player_position(
    _: Trigger<ResetPlayerPosition>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}
