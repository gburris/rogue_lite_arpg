use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use serde::Deserialize;
use serde_json::{Value, json};

use super::json::{debug_name, parse_params};
use crate::prelude::*;

/// `game/start`: leaves the start screen, as its button does.
#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
pub(super) fn start_game(
    In(_): In<Option<Value>>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) -> BrpResult {
    if *app_state.get() == AppState::StartScreen {
        next_app_state.set(AppState::AssetLoading);
    }

    Ok(json!({ "state": debug_name(app_state.get()) }))
}

#[derive(Deserialize)]
struct LoadMapParams {
    recipe: String,
    seed: Option<u64>,
}

/// `game/load_map {recipe, seed?}`: builds a map recipe by name, replacing the current map.
pub(super) fn load_map(In(params): In<Option<Value>>, mut next_map: NextMap) -> BrpResult {
    let LoadMapParams { recipe, seed } = parse_params(params)?;

    next_map
        .set(MapRequest::Named { recipe, seed })
        .map_err(BrpError::internal)?;

    Ok(Value::Null)
}

#[derive(Deserialize)]
struct TimeParams {
    paused: Option<bool>,
    speed: Option<f32>,
}

/// `game/time {paused?, speed?}`: pauses, resumes or rescales game time, then returns it. Physics
/// and fixed steps follow; menus and real-time tools don't.
pub(super) fn control_time(
    In(params): In<Option<Value>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) -> BrpResult {
    let TimeParams { paused, speed } = parse_params(params)?;

    match paused {
        Some(true) => virtual_time.pause(),
        Some(false) => virtual_time.unpause(),
        None => {}
    }
    if let Some(speed) = speed {
        virtual_time.set_relative_speed(speed);
    }

    Ok(json!({
        "time": virtual_time.elapsed_secs(),
        "paused": virtual_time.is_paused(),
        "speed": virtual_time.relative_speed(),
    }))
}

#[derive(Deserialize)]
struct ScreenshotParams {
    path: String,
}

/// `game/screenshot {path}`: saves the primary window to an image file a frame or two later.
pub(super) fn take_screenshot(In(params): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let ScreenshotParams { path } = parse_params(params)?;

    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));

    Ok(Value::Null)
}

/// `game/exit`: closes the game after this frame.
#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
pub(super) fn exit_game(
    In(_): In<Option<Value>>,
    mut app_exit: MessageWriter<AppExit>,
) -> BrpResult {
    app_exit.write(AppExit::Success);

    Ok(Value::Null)
}
