use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::plugins::{EnemySpawnerPlugin, PlayerPlugin, ProjectilePlugin, UIPlugin};
use game_dev_project::resources::{MapBounds, TileSize, PlayerSize, ProcessedProjectiles};
use bevy::utils::HashSet;

fn main() {
    let tile_size_x = 16.0;
    let tile_size_y = 16.0;
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
        .add_plugins(EnemySpawnerPlugin)
        .add_plugins(ProjectilePlugin)
        .add_plugins(UIPlugin)
        .add_systems(Startup, game_dev_project::systems::generate_tilemap)
        .insert_resource(TileSize{  
            x: 16.0,
            y: 16.0
        })
        .insert_resource(MapBounds {
            min_x: -100.0 * tile_size_x,
            min_y: -100.0 * tile_size_y,
            max_x: 100.0 * tile_size_x,
            max_y: 100.0 * tile_size_y,
        })
        .insert_resource(PlayerSize {
            x: 256.0,
            y: 256.0,
        })
        .insert_resource(ProcessedProjectiles {
            set: HashSet::new(),
        })
        .run();
}
