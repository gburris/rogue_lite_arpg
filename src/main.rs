use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use plugins::PlayerPlugin;

mod components; 
mod helpers;
mod plugins;
mod systems; 

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
        primary_window: Some(Window {
            title: String::from(
                   "Basic Example 2 - Press Space to change Texture and H to show/hide tilemap.",
            ),
            ..Default::default()
        }),
        ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, systems::generate_tilemap)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
