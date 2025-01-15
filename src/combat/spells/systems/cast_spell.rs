use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    combat::spells::{components::Spell, spell_factory::SpellFactory},
    configuration::assets::SpriteAssets,
    player::components::Player,
};

pub fn cast_spell_system(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    sprites: Res<SpriteAssets>,
    player: Query<&Transform, With<Player>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if let Some(target_transform) = calculate_cast_position_and_angle(player, window, camera_query)
    {
        if buttons.just_pressed(MouseButton::Left) {
            SpellFactory::spawn_spell(
                &mut commands,
                Spell::Fireball,
                target_transform,
                &sprites,
                &mut texture_atlas_layouts,
            );
        }
        if buttons.just_pressed(MouseButton::Right) {
            SpellFactory::spawn_spell(
                &mut commands,
                Spell::Icebolt,
                target_transform,
                &sprites,
                &mut texture_atlas_layouts,
            );
        }
    }
}

fn calculate_cast_position_and_angle(
    player: Query<&Transform, With<Player>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) -> Option<Transform> {
    let mut player_transform: &Transform = &Transform::default();

    for transform in &player {
        player_transform = transform;
    }
    let player_pos = player_transform.translation;
    let mut target_transform = Transform::from_xyz(player_pos.x, player_pos.y, 0.5);

    target_transform.rotation = player_transform.rotation;
    let (camera, camera_transform) = *camera_query;

    let Ok(window) = window.get_single() else {
        return None;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return None;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return None;
    };

    let player_dir = player_transform.local_x().truncate();
    let cursor_dir = point - player_transform.translation.truncate();
    let angle = player_dir.angle_to(cursor_dir);

    target_transform.rotate_z(angle);

    Some(target_transform)
}
