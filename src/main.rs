use bevy::color::palettes::basic::WHITE;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Define the Player component
#[derive(Component)]
struct Player {
    speed: f32,
    position: Position,
}

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Player {
            speed: 10.0,
            position: Position { x: 100.0, y: 100.0 },
        },
        Sprite::from_image(asset_server.load("skeleton.png")),
        Transform::from_xyz(0., 0., 0.),
    ));
}

// Define the plugin to organize player-related functionality
struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (player_movement, face_cursor_system, camera_follow_system, draw_cursor),
        );
    }
}

fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // Match the camera's position to the player's position (retain the z-depth)
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}
// System for player movement
fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        player.position.x += direction.x * player.speed;
        player.position.y += direction.y * player.speed;
        transform.translation = Vec3::new(player.position.x, player.position.y, 0.0);
    }
}

fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    gizmos.circle_2d(point, 10., WHITE);
}

fn face_cursor_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    // Get the primary window
    if let Ok(window) = windows.get_single() {
        // Get the cursor position in screen space
        if let Some(cursor_position) = window.cursor_position() {
            let screen_center_x = window.width() / 2.0;

            // Update the player's transform to face the cursor
            for mut transform in query.iter_mut() {
                if cursor_position.x < screen_center_x {
                    // Cursor is on the left side of the screen
                    transform.scale.x = 1.0; // Flip sprite to face left
                } else {
                    // Cursor is on the right side of the screen
                    transform.scale.x = -1.0; // Face right
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .run();
}
