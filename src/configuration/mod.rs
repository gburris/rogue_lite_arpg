use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use properties::PropertiesPlugin;

use crate::{
    ai::AIPlugin,
    animation::AnimationPlugin,
    combat::plugin::CombatPlugin,
    despawn::plugin::DespawnPlugin,
    economy::EconomyPlugin,
    enemy::plugin::EnemyPlugin,
    items::{equipment::EquipmentPlugin, lootable::plugin::LootablePlugin},
    labels::{
        sets::{InGameSet, MainSet},
        states::{AppState, PausedState, PlayingState},
    },
    map::plugin::MapPlugin,
    npc::NPCPlugin,
    player::plugin::PlayerPlugin,
    progression::plugin::ProgressionPlugin,
    ui::plugin::UIPlugin,
};

mod properties;
// Re-export essential components/constants
pub mod assets;
pub mod collision;
pub mod time;
pub mod view;

pub use collision::GameCollisionLayer;
pub use view::{shadow, YSort, ZLayer, CHARACTER_FEET_POS_OFFSET};

// Setup plugin - primary configuration plugin
pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
        {
            #[path = "debug.rs"]
            mod debug;
            app.add_plugins(debug::DebugPlugin);
        }

        #[cfg(not(debug_assertions))]
        app.add_plugins(
            DefaultPlugins
                .set(view::get_window_plugin())
                .set(ImagePlugin::default_nearest()),
        );

        app
            // initialize states
            .init_state::<AppState>()
            .add_sub_state::<PausedState>()
            .add_sub_state::<PlayingState>()
            .add_systems(Startup, view::spawn_camera)
            // avian recommendeds ordering camera following logic in PostUpdate after transform prop
            .add_systems(
                PostUpdate,
                view::camera_follow_system.before(TransformSystem::TransformPropagate),
            )
            .add_systems(FixedUpdate, view::ysort_transforms.in_set(MainSet::InGame));
    }
}

// Schedule plugin - configures system sets and ordering
pub struct SchedulePlugin;
impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MainSet::InGame.run_if(in_state(AppState::Playing)),
                MainSet::Menu.run_if(in_state(AppState::Paused)),
                MainSet::Shared,
            )
                .chain()
                .after(LoadingStateSet(AppState::AssetLoading)), // appease the system ordering gods
        )
        // Configuring the ordering of our gameplay loop using these main sets:
        // Despawn Entitites -> Handle Input -> Simulation -> Update HUD / overlay UI -> Physics -> Collision
        .configure_sets(
            Update,
            // since 0.13, apply_deferred is automatically applied when a command is run in a system
            // this ensures entities are always despawned before this frames simulation runs
            (
                InGameSet::DespawnEntities,
                InGameSet::PlayerInput,
                InGameSet::Simulation,
                InGameSet::Collision,
                InGameSet::Vfx,
                InGameSet::HudOverlay,
            )
                .chain()
                .in_set(MainSet::InGame),
        )
        .configure_sets(
            FixedUpdate,
            MainSet::InGame.run_if(in_state(AppState::Playing)),
        );
    }
}

// Game plugins - combines all plugins together
pub struct GamePlugins;

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity::ZERO) // no gravity since this is top-down game
            // Setup and configuration
            .add_plugins((
                PropertiesPlugin,
                SetupPlugin,
                AnimationPlugin,
                SchedulePlugin,
            ))
            // Third-party plugins
            .add_plugins((assets::AssetLoadingPlugin, TilemapPlugin))
            // Core systems
            .add_plugins((
                PhysicsPlugins::default().with_length_unit(32.0), // 32 pixels per meter
                DespawnPlugin,
                AIPlugin,
                CombatPlugin,
                ProgressionPlugin,
                EconomyPlugin,
                EquipmentPlugin,
            ))
            // Entity systems
            .add_plugins((
                MapPlugin,
                LootablePlugin,
                PlayerPlugin,
                EnemyPlugin,
                NPCPlugin,
            ))
            // UI
            .add_plugins(UIPlugin);
    }
}

// NOTE: Eventually, as we add more plugins, these might need to load platform-specific
// plugins or resources. For now, we can just use the same plugin for all platforms.
//
// We use apprt as the shared module name, and conditionally compile platform-specific code.
// And now we only have to declare the cfg once at the top of the module.
pub struct AppRuntimePlugin;

#[cfg(target_arch = "wasm32")]
mod apprt {
    use super::*;
    impl Plugin for AppRuntimePlugin {
        // Add any WASM-specific plugins here
        fn build(&self, app: &mut App) {
            app.add_plugins(GamePlugins);
        }
    }
}
mod apprt {
    use super::*;
    impl Plugin for AppRuntimePlugin {
        // Add native-only plugins
        fn build(&self, app: &mut App) {
            app.add_plugins(GamePlugins);
        }
    }
}
