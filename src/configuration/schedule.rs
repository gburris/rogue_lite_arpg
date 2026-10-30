use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateSet;

pub(super) fn plugin(app: &mut App) {
    // initialize states
    app.init_state::<AppState>().add_sub_state::<PlayingState>();

    // Ensure InGame system only runs when not paused
    app.configure_sets(
        Update,
        (
            MainSystems::InGame
                .run_if(in_state(Pause(false)).and_then(in_state(AppState::Playing))),
            MainSystems::Menu.run_if(in_state(Pause(true))),
            MainSystems::Shared,
        )
            .chain()
            .after(LoadingStateSet(AppState::AssetLoading)), // appease the system ordering gods
    );

    // Must order per-schedule, so same as above but for FixedUpdate systems
    app.configure_sets(
        FixedUpdate,
        MainSystems::InGame.run_if(in_state(Pause(false)).and_then(in_state(AppState::Playing))),
    );

    // Configuring the ordering of our gameplay loop using these main sets
    // Helps set command flush break-points and reduce system ambiguities
    app.configure_sets(
        Update,
        (
            InGameSystems::DespawnEntities,
            InGameSystems::PlayerInput,
            InGameSystems::Simulation,
            InGameSystems::Collision,
            InGameSystems::Vfx,
            InGameSystems::HudOverlay,
        )
            .chain()
            .in_set(MainSystems::InGame),
    );

    app.configure_sets(
        Update,
        (SimulationSystems::Logic, SimulationSystems::Derive)
            .chain()
            .in_set(InGameSystems::Simulation),
    );

    // Setup Pausing!
    app.init_state::<Pause>()
        .add_systems(OnEnter(Pause(true)), pause_game)
        .add_systems(OnEnter(Pause(false)), resume_game);
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MainSystems {
    InGame,
    Menu,
    Shared,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum InGameSystems {
    DespawnEntities, // Despawn entities only! MUST happen before simulation of this new frame we are in!
    PlayerInput,
    Simulation, // Most game logic (queries modifying components)
    Collision,  // Finally detect collisions using avian based on velocity changed
    Vfx,        // Any visual change that should not affect physics or collisions
    HudOverlay, // Render UI overlay based on simulation
}

/// Ordered phases within [`InGameSystems::Simulation`].
///
/// Producers that may start or stop equipment use (directly, or via queued
/// `StartUsingEquipment` / `StopUsingEquipment` observers, which flush between
/// systems) belong in `Logic`. State derived from the `EquipmentInUse` marker, such as
/// attack animation state, belongs in `Derive` and is guaranteed to see every
/// same-frame use and cancellation.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SimulationSystems {
    /// Game logic that may queue equipment use or cancellation.
    Logic,
    /// State derived from simulation results.
    Derive,
}

#[derive(States, Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub enum AppState {
    #[default]
    StartScreen,
    AssetLoading,
    SpawnMap,    // Used to 1. put a load screen on 2. Spawn everything in the map
    SpawnPlayer, // Also creates player overlay UI
    Playing,
    GameOver,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Pause(pub bool);

#[derive(SubStates, Eq, Hash, Default, Clone, Copy, Debug, PartialEq)]
#[source(AppState = AppState::Playing)]
pub enum PlayingState {
    #[default]
    Playing,
    Death,
}

#[derive(Event)]
pub struct RestartEvent {
    pub player_level: u32,
}

// Make pause menu visible when we enter the state
fn resume_game(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

// Cleanup pause menu once we return to game, set it to hidden
fn pause_game(mut time: ResMut<Time<Physics>>) {
    time.pause();
}
