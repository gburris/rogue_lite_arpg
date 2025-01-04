use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use rand::Rng;

use crate::{
    labels::states::GameState,
    map::components::WarpZone,
    resources::{assets::SpriteAssets, CurrentZoneLevel, MapBounds},
};

pub fn warpzone_setup(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    mut game_state: ResMut<NextState<GameState>>,
    mut zone_level: ResMut<CurrentZoneLevel>,
    map_bounds: Res<MapBounds>,
) {
    // Move warpzone to a random position on the right side of the map
    let warpzone_position = Vec3::new(
        map_bounds.max_x - 100.0,
        rand::thread_rng().gen_range(-200.0..200.0),
        1.0,
    );

    zone_level.0 += 1;

    warn!("Progressing to level: {:?}", zone_level.0);

    commands.spawn((
        WarpZone::default(), //Default is LevelTwo, Since we begin on LevelOne
        RigidBody::Static,
        Collider::rectangle(100.0, 100.0),
        Sprite::from_image(sprites.warp_zone.clone()),
        Transform::from_translation(warpzone_position),
    ));
    game_state.set(GameState::Playing);
}
