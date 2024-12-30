use bevy::{prelude::*, window::PrimaryWindow};

use crate::components::{Fireball, Player};

pub fn cast_fireball(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
    player: Query<&Transform, With<Player>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    let mut player_transform: &Transform = &Transform::default();

    for transform in &player {
        player_transform = transform;
    }
    let player_pos = player_transform.translation;
    let mut target_transform = Transform::from_xyz(player_pos.x, player_pos.y, 0.5);

    target_transform.rotation = player_transform.rotation;
    let (camera, camera_transform) = *camera_query;

    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let player_dir = player_transform.local_x().truncate();
    let cursor_dir = point - player_transform.translation.truncate();
    let angle = player_dir.angle_to(cursor_dir);

    target_transform.rotate_z(angle);

    if buttons.just_pressed(MouseButton::Left) {
        commands.spawn((
            Fireball { speed: 300. },
            Sprite::from_image(asset_server.load("fireball/FB001.png")),
            target_transform,
        ));
    }
}
