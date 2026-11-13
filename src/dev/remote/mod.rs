mod explain;
mod input;
mod json;
mod session;
mod setup;
mod snapshot;

use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy::remote::http::RemoteHttpPlugin;

/// Serves the Bevy Remote Protocol on `127.0.0.1:15702`, adding `game/…` methods to the built-in
/// `world.…` ones so a script can play the game and read it back. Each handler's doc lists its
/// params, e.g. `curl -d '{"jsonrpc":"2.0","id":1,"method":"game/snapshot"}' 127.0.0.1:15702`.
pub(super) fn plugin(app: &mut App) {
    let remote_plugin = RemotePlugin::default()
        .with_method_main("game/start", session::start_game)
        .with_method_main("game/load_map", session::load_map)
        .with_method_main("game/time", session::control_time)
        .with_method_main("game/screenshot", session::take_screenshot)
        .with_method_main("game/exit", session::exit_game)
        .with_method_main("game/snapshot", snapshot::take_snapshot)
        .with_method_main("game/explain", explain::explain_brains)
        .with_method_main("game/ui", input::list_ui_text)
        .with_method_main("game/player_input", input::set_player_input)
        .with_method_main("game/key", input::press_key)
        .with_method_main("game/click", input::click)
        .with_method_main("game/teleport_player", setup::teleport_player)
        .with_method_main("game/spawn_enemies", setup::spawn_enemies)
        .with_method_main("game/despawn_enemies", setup::despawn_enemies)
        .with_method_main("game/set_player_health", setup::set_player_health)
        .with_method_main("game/set_player_mana", setup::set_player_mana)
        .with_method_main("game/set_invincible", setup::set_invincible)
        .with_method_main("game/give_item", setup::give_item)
        .with_method_main("game/use_item", setup::use_item);

    app.add_plugins((
        remote_plugin,
        RemoteHttpPlugin::default(),
        input::plugin,
        snapshot::plugin,
    ));
}
