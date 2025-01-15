use bevy::prelude::*;
use rand::Rng;

use crate::{
    configuration::assets::SpriteAssets,
    labels::states::AppState,
    map::{
        components::Portal,
        resources::{CurrentZoneLevel, MapBounds},
    },
};

pub fn warpzone_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    mut zone_level: ResMut<CurrentZoneLevel>,
    sprites: Res<SpriteAssets>,
    map_bounds: Res<MapBounds>,
) {
    // Move warpzone to a random position on the right side of the map
    let mut rng = rand::thread_rng();
    let y_radius = map_bounds.max_y - 200.0;
    let warpzone_position = Vec3::new(
        map_bounds.max_x - rng.gen_range(100.0..300.0),
        rng.gen_range(-y_radius..y_radius),
        1.0,
    );

    zone_level.0 += 1;

    commands.spawn((
        Portal::WarpZone,
        Sprite::from_image(sprites.warp_zone.clone()),
        Transform::from_translation(warpzone_position),
    ));
    game_state.set(AppState::Playing);
}
