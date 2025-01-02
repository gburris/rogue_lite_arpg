use avian2d::prelude::{Gravity, PhysicsDebugPlugin};
use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::plugins::{
    EnemyPlugin, ExperiencePlugin, LevelPlugin, PlayerPlugin, ProjectilePlugin,
};
use game_dev_project::resources::{PlayerSize, ProcessedProjectiles, ProcessedWarpZoneEvents};

fn main() {
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
        .add_plugins(LevelPlugin)
        .add_plugins(ExperiencePlugin)
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .insert_resource(ProcessedProjectiles {
            set: HashSet::new(),
        })
        .insert_resource(ProcessedWarpZoneEvents {
            set: HashSet::new(),
        })
        .run();
}
