use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateSet;

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MainSystems::InGame.run_if(in_state(AppState::Playing)),
                MainSystems::Menu.run_if(in_state(AppState::Paused)),
                MainSystems::Shared,
            )
                .chain()
                .after(LoadingStateSet(AppState::AssetLoading)), // appease the system ordering gods
        )
        // Configuring the ordering of our gameplay loop using these main sets:
        // Despawn Entitites -> Handle Input -> Simulation -> Update HUD / overlay UI -> Physics -> Collision
        .configure_sets(
            Update,
            (
                // Since 0.13, apply_deferred is automatically applied when a command is run in a system
                // This ensures entities are always despawned before this frames simulation runs
                InGameSystems::DespawnEntities,
                InGameSystems::PlayerInput,
                InGameSystems::Simulation,
                InGameSystems::Collision,
                InGameSystems::Vfx,
                InGameSystems::HudOverlay,
            )
                .chain()
                .in_set(MainSystems::InGame),
        )
        .configure_sets(
            FixedUpdate,
            MainSystems::InGame.run_if(in_state(AppState::Playing)),
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MainSystems {
    InGame,
    Menu,
    Shared,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InGameSystems {
    DespawnEntities, // Despawn entities only! MUST happen before simulation of this new frame we are in!
    PlayerInput,
    Simulation, // Most game logic (queries modifying components)
    Collision,  // Finally detect collisions using avian based on velocity changed
    Vfx,        // Any visual change that should not affect physics or collisions
    HudOverlay, // Render UI overlay based on simulation
}

#[derive(States, Clone, Eq, PartialEq, Default, Debug, Hash)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    StartScreen,
    AssetLoading,
    CreateHub,
    SpawnPlayer, // Also creates player overlay UI
    SpawnZone,   //Used to 1. put a load screen on 2. Spawn everything in ur zone
    Playing,
    Paused,
    GameOver,
}

#[derive(SubStates, Eq, Default, Hash, Clone, Copy, Debug, PartialEq)]
#[source(AppState = AppState::Paused)]
#[states(scoped_entities)]
pub enum PausedState {
    #[default]
    MainMenu,
    Inventory,
    StatsShop,
    ItemsShop,
    Stats,
}

#[derive(SubStates, Eq, Hash, Default, Clone, Copy, Debug, PartialEq)]
#[source(AppState = AppState::Playing)]
pub enum PlayingState {
    #[default]
    Playing,
    Death,
}
