//! - [`nc`] - messages / data structures passed over connection between client and server. mostly Ron objects
//! - [`plugin`] - exports a `NetConsolePlugin` that opens a TCP listener for `127.0.0.1:8080` on bevy Startup.
//!     -  TODO: configuration
//!
//! - [`plugin::handlers`] - the bevy systems responsible for managing and
//!   coordinating communication between the bevy app and the requesting client.
//!
//! - [`plugin::NetConsolePlugin`]

pub mod nc;
pub mod plugin;
pub use plugin::NetConsolePlugin;
pub(crate) mod bevy {
    pub(crate) use {
        bevy_ecs as ecs, bevy_ptr as ptr, bevy_reflect as reflect, bevy_tasks as tasks, bevy_utils as utils,
    };
    pub(crate) mod prelude {
        pub(crate) use {bevy_app::prelude::*, bevy_ecs::prelude::*};
    }
}
