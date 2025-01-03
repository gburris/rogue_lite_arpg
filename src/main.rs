use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::enemy::plugins::EnemyPlugin;
use game_dev_project::player::plugins::{ExperiencePlugin, PlayerPlugin};
use game_dev_project::plugins::{LevelPlugin, ProjectilePlugin};
use game_dev_project::resources::assets::AssetLoadingPlugin;
use game_dev_project::resources::PlayerSize;
use game_dev_project::schedule::SchedulePlugin;
use game_dev_project::setup::SetupPlugin;

fn main() {
    App::new()
        .add_plugins((
            SetupPlugin,
            SchedulePlugin,
            AssetLoadingPlugin,
            TilemapPlugin,
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            LevelPlugin,
            ExperiencePlugin,
        ))
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .run();
}
