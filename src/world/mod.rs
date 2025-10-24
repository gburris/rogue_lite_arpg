mod chest;
mod gold;

use bevy::prelude::*;

pub mod prelude {
    pub use super::chest::*;
    pub use super::gold::*;
}

pub fn plugin(app: &mut App) {
    app.add_plugins((chest::plugin, gold::plugin));
}
