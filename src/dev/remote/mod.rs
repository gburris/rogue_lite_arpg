mod explain;
mod input;
mod methods;
mod session;
mod setup;
mod snapshot;

use bevy::prelude::*;
use bevy::remote::http::{HostPort, RemoteHttpPlugin};
use bevy::render::RenderApp;
use bevy::winit::WinitSettings;

use super::game_args::GameArgs;

/// How far above `--remote-port` the render sub-app's remote server listens.
pub(super) const RENDER_PORT_OFFSET: u16 = 1000;

/// Serves the Bevy Remote Protocol on `127.0.0.1`, at `--remote-port` (15702 unless given),
/// adding the `game/…` methods below to the built-in `world.…` ones so a script can play the game
/// and read it back, e.g. `curl -d '{"jsonrpc":"2.0","id":1,"method":"game/help"}'
/// 127.0.0.1:15702`.
pub(super) fn plugin(app: &mut App) {
    let game_args = *app
        .world()
        .get_resource::<GameArgs>()
        .expect("main inserts GameArgs before adding plugins");

    let (remote_plugin, method_help) = methods::game_methods().into_plugin_and_help();

    app.insert_resource(method_help).add_plugins((
        remote_plugin,
        RemoteHttpPlugin::default().with_port(game_args.remote_port),
        input::plugin,
        snapshot::plugin,
    ));
    // The plugin also serves the render sub-app, on a fixed port that only one of several games
    // running at once can take; derive it from the game's own port instead.
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.insert_resource(HostPort(game_args.remote_port + RENDER_PORT_OFFSET));
    }

    if game_args.update_while_unfocused {
        app.insert_resource(WinitSettings::continuous());
    }
}
