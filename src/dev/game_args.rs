//! The game's command line.

use bevy::prelude::*;
use clap::Parser;

use super::stress::StressArgs;

/// The game's command line. `main` parses it before building the app, so `--help` and bad flags
/// exit before a window opens.
#[derive(Resource, Parser, Clone, Copy, Debug)]
#[command(about = "Baba Yaga development flags.")]
pub(crate) struct GameArgs {
    #[command(flatten)]
    pub(super) stress: StressArgs,

    #[cfg(feature = "dev")]
    #[arg(
        long,
        help = "Write the Update and FixedUpdate schedule graphs to runs/schedules, then exit"
    )]
    pub(crate) dump_schedules: bool,

    #[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
    #[arg(
        long,
        default_value_t = bevy::remote::http::DEFAULT_PORT,
        help = "Port the remote protocol server listens on, so several games can run at once"
    )]
    pub(super) remote_port: u16,
}
