//! Development tooling: debug rendering and inspection, the F3 overlay, `game/…` diagnostics,
//! Tracy plots, hitch marking and detection, time controls, cheat keys, the stress arena and the
//! schedule dump. Compiled only with the `dev_tools` feature, and never needed to boot the game.

mod build_info;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
mod cheats;
#[cfg(feature = "dev")]
mod debug;
mod diagnostics;
mod game_args;
#[cfg(not(target_arch = "wasm32"))]
mod hitch;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
mod hud;
#[cfg(feature = "dev")]
mod schedule_dump;
mod stress;
#[cfg(feature = "dev")]
mod time_control;
#[cfg(feature = "tracy")]
mod tracy;

use bevy::prelude::*;

pub(super) use self::game_args::GameArgs;
#[cfg(feature = "dev")]
pub(super) use self::schedule_dump::dump_schedules;

pub mod prelude {
    pub use super::stress::StressArena;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(diagnostics::plugin);

    #[cfg(feature = "dev")]
    app.add_plugins((debug::plugin, time_control::plugin));

    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins((cheats::plugin, hud::plugin));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(hitch::plugin);

    // `bevy/trace_tracy` makes the render plugin add this itself.
    #[cfg(all(feature = "dev", not(feature = "tracy")))]
    app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);

    app.add_plugins(stress::plugin);

    #[cfg(feature = "tracy")]
    app.add_plugins(tracy::plugin);
}
