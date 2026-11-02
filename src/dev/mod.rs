//! Development tooling: debug rendering and inspection, the F3 overlay, `game/…` diagnostics,
//! Tracy plots, the F4 hitch marker, the stress arena and the schedule dump. Compiled only for the
//! `dev`, `stress` and `profile` features, and never needed to boot the game.

#[cfg(feature = "dev")]
mod debug;
mod diagnostics;
#[cfg(all(any(feature = "dev", feature = "profile"), not(target_arch = "wasm32")))]
mod hitch;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
mod hud;
#[cfg(feature = "dev")]
mod schedule_dump;
#[cfg(feature = "stress")]
mod stress;
#[cfg(feature = "profile")]
mod tracy;

use bevy::prelude::*;

#[cfg(feature = "dev")]
pub use self::schedule_dump::dump_schedules_if_requested;

pub mod prelude {
    #[cfg(feature = "stress")]
    pub use super::stress::StressArena;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(diagnostics::plugin);

    #[cfg(feature = "dev")]
    app.add_plugins(debug::plugin);

    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(hud::plugin);

    #[cfg(all(any(feature = "dev", feature = "profile"), not(target_arch = "wasm32")))]
    app.add_plugins(hitch::plugin);

    // `bevy/trace_tracy` makes the render plugin add this itself.
    #[cfg(all(feature = "dev", not(feature = "profile")))]
    app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);

    #[cfg(feature = "stress")]
    app.add_plugins(stress::plugin);

    #[cfg(feature = "profile")]
    app.add_plugins(tracy::plugin);
}
