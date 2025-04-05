pub mod net;
pub mod plugin;

pub mod bevy {
    pub use {
        bevy_app as app, bevy_ecs as ecs, bevy_ptr as ptr, bevy_reflect as reflect, bevy_tasks as tasks,
        bevy_utils as utils,
    };
    pub mod prelude {
        pub use {
            bevy_app::prelude::*, bevy_ecs::prelude::*, bevy_reflect::prelude::*, bevy_tasks::prelude::*,
            bevy_utils::prelude::*,
        };
    }
}
