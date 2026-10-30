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
        (
            SimulationSystems::Upkeep,
            SimulationSystems::Perceive,
            SimulationSystems::Act,
            SimulationSystems::Derive,
        )
            .chain()
            .in_set(InGameSystems::Simulation),
    );

    app.configure_sets(
        Update,
        (VfxSystems::Select, VfxSystems::Animate, VfxSystems::Tint)
            .chain()
            .in_set(InGameSystems::Vfx),
    );

    // Setup Pausing!
    app.init_state::<Pause>()
        .add_systems(OnEnter(Pause(true)), pause_game)
        .add_systems(OnEnter(Pause(false)), resume_game);
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MainSystems {
    InGame,
    Menu,
    Shared,
}

/// Ordered phases of a gameplay frame. Every gameplay system belongs to exactly one, or to a
/// sub-phase nested inside one.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InGameSystems {
    /// Despawns only, before this frame's simulation sees the entities.
    DespawnEntities,
    PlayerInput,
    /// Game logic, split into [`SimulationSystems`].
    Simulation,
    /// Reacts to collisions Avian detected from the simulated velocities.
    Collision,
    /// Visual changes that never affect physics or collisions, split into [`VfxSystems`].
    Vfx,
    /// UI overlay drawn from the simulation results.
    HudOverlay,
}

/// Ordered phases within [`InGameSystems::Simulation`].
///
/// Producers that may start or stop equipment use (directly, or via queued
/// `StartUsingEquipment` / `StopUsingEquipment` observers, which flush between
/// systems) run before `Derive`. State derived from the `EquipmentInUse` marker, such as
/// attack animation state, belongs in `Derive` and is guaranteed to see every
/// same-frame use and cancellation.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimulationSystems {
    /// Advances ongoing state: regeneration, cooldowns, status effects, in-progress swings and
    /// shield drain.
    Upkeep,
    /// Updates who sees and aims at what.
    Perceive,
    /// Decides and acts on this frame's perception: AI decisions, motion and equipment use.
    Act,
    /// State derived from simulation results.
    Derive,
}

/// Ordered phases within [`InGameSystems::Vfx`].
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VfxSystems {
    /// Chooses which animation each sprite plays.
    Select,
    /// Advances animations and other motion-only effects.
    Animate,
    /// Overrides sprite colors, such as hit flashes, on top of the animated frame.
    Tint,
}

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pause(pub bool);

#[derive(SubStates, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Playing)]
pub enum PlayingState {
    #[default]
    Playing,
    Death,
}

#[derive(Event)]
pub struct Restart {
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
