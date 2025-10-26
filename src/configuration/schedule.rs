use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingStateSet;

use crate::{character::player::PauseInputEvent, prelude::CleanupZone};

pub(super) fn plugin(app: &mut App) {
    // initialize states
    app.init_state::<AppState>()
        .init_state::<Menu>()
        .add_sub_state::<PlayingState>();

    // Ensure InGame system only runs when not paused
    app.configure_sets(
        Update,
        (
            MainSystems::InGame.run_if(in_state(Pause(false)).and(in_state(AppState::Playing))),
            MainSystems::Menu.run_if(in_state(Pause(true))),
            MainSystems::Shared,
        )
            .chain()
            .after(LoadingStateSet(AppState::AssetLoading)), // appease the system ordering gods
    );

    // Must order per-schedule, so same as above but for FixedUpdate systems
    app.configure_sets(
        FixedUpdate,
        MainSystems::InGame.run_if(in_state(Pause(false)).and(in_state(AppState::Playing))),
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

    // Setup Pausing!
    app.init_state::<Pause>()
        .add_systems(OnEnter(Pause(true)), pause_game)
        .add_systems(OnEnter(Pause(false)), resume_game)
        .add_observer(on_pause_input);

    app.add_systems(OnEnter(AppState::Transition), transition_zones);
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
pub enum AppState {
    #[default]
    StartScreen,
    AssetLoading,
    CreateHub,
    SpawnPlayer, // Also creates player overlay UI
    SpawnZone,   //Used to 1. put a load screen on 2. Spawn everything in ur zone
    Playing,
    Transition,
    GameOver,
}

#[derive(States, Eq, Default, Hash, Clone, Copy, Debug, PartialEq)]
pub enum Menu {
    #[default]
    None, // In Game
    MainMenu,
    Inventory,
    StatsShop,
    ItemsShop,
    Stats,
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
    debug!("resume_game");
    time.unpause();
}

// Cleanup pause menu once we return to game, set it to hidden
fn pause_game(mut time: ResMut<Time<Physics>>) {
    debug!("pause_game");
    time.pause();
}

fn on_pause_input(
    pause_input_trigger: On<PauseInputEvent>,
    pause: Res<State<Pause>>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut next_pause: ResMut<NextState<Pause>>,
) {
    match pause.get() {
        Pause(true) => {
            debug!("Currently paused, transitioning to playing");
            next_menu.set(Menu::None);
            next_pause.set(Pause(false));
        }
        Pause(false) => {
            debug!("Not currently paused, transitioning to paused");
            next_pause.set(Pause(true));

            if let Some(menu) = pause_input_trigger.menu {
                next_menu.set(menu);
            } else {
                next_menu.set(Menu::MainMenu);
            }
        }
    }
}

fn transition_zones(mut commands: Commands, mut game_state: ResMut<NextState<AppState>>) {
    commands.trigger(CleanupZone);
    game_state.set(AppState::SpawnZone);
}
