mod explain;
mod input;
mod methods;
mod session;
mod setup;
mod snapshot;

use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;

use super::game_args::GameArgs;

/// Serves the Bevy Remote Protocol on `127.0.0.1`, at `--remote-port` (15702 unless given),
/// adding the `game/…` methods below to the built-in `world.…` ones so a script can play the game
/// and read it back, e.g. `curl -d '{"jsonrpc":"2.0","id":1,"method":"game/help"}'
/// 127.0.0.1:15702`.
pub(super) fn plugin(app: &mut App) {
    let remote_port = app
        .world()
        .get_resource::<GameArgs>()
        .expect("main inserts GameArgs before adding plugins")
        .remote_port;

    let (remote_plugin, method_help) = methods::game_methods().into_plugin_and_help();

    app.insert_resource(method_help).add_plugins((
        remote_plugin,
        RemoteHttpPlugin::default().with_port(remote_port),
        input::plugin,
        snapshot::plugin,
    ));
}
