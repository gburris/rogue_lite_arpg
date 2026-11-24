//! The game's command line.

use bevy::prelude::*;
use clap::Parser;

#[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
use super::remote::RENDER_PORT_OFFSET;
use super::stress::StressArgs;

/// The highest `--remote-port` that leaves room for the render server's port above it.
#[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
const HIGHEST_REMOTE_PORT: i64 = (u16::MAX - RENDER_PORT_OFFSET) as i64;

/// The game's command line. `main` parses it before building the app, so `--help` and bad flags
/// exit before a window opens.
#[derive(Resource, Parser, Clone, Copy, Debug)]
#[command(about = "Baba Yaga development flags.")]
pub struct GameArgs {
    #[command(flatten)]
    pub(super) stress: StressArgs,

    #[cfg(feature = "dev")]
    #[arg(
        long,
        help = "Write the Update and FixedUpdate schedule graphs to runs/schedules, then exit"
    )]
    pub dump_schedules: bool,

    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    #[arg(
        long,
        value_parser = clap::value_parser!(u32).range(1..),
        help = "Cap the frame rate, to check that gameplay doesn't depend on it"
    )]
    pub(super) max_fps: Option<u32>,

    #[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
    #[arg(
        long,
        default_value_t = bevy::remote::http::DEFAULT_PORT,
        value_parser = clap::value_parser!(u16).range(..=HIGHEST_REMOTE_PORT),
        help = "Port the remote protocol server listens on, so several games can run at once; \
                the render server listens a fixed step above it"
    )]
    pub(super) remote_port: u16,

    #[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
    #[arg(
        long,
        help = "Keep updating every frame without focus, so a script plays at full speed while \
                the window is in the background"
    )]
    pub(super) update_while_unfocused: bool,
}
