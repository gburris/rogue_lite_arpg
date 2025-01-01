use avian2d::prelude::{Gravity, PhysicsDebugPlugin};
use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::plugins::{EnemyPlugin, PlayerPlugin, ProjectilePlugin};
use game_dev_project::resources::{MapBounds, PlayerSize, ProcessedProjectiles, TileSize};

fn main() {
    let tile_size_x = 16.0;
    let tile_size_y = 16.0;
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from(
                            "Right click to cast Icebolt Left Click to Cast Fireball",
                        ),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .insert_resource(Gravity::ZERO)
        .add_plugins(TilemapPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(ProjectilePlugin)
        .add_systems(Startup, game_dev_project::systems::generate_tilemap)
        .insert_resource(TileSize { x: 16.0, y: 16.0 })
        .insert_resource(MapBounds {
            min_x: -100.0 * tile_size_x,
            min_y: -100.0 * tile_size_y,
            max_x: 100.0 * tile_size_x,
            max_y: 100.0 * tile_size_y,
        })
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .insert_resource(ProcessedProjectiles {
            set: HashSet::new(),
        })
        .run();
}
