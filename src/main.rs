use bevy::prelude::*;
use plugins::PlayerPlugin;

mod components;  // Declare the components module
mod systems; //Declare the systems module
mod plugins;


// Define the plugin to organize player-related functionality

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
        primary_window: Some(Window {
            title: String::from(
                   "Basic Example - Press Space to change Texture and H to show/hide tilemap.",
            ),
            ..Default::default()
        }),
        ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(PlayerPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .run();
}
