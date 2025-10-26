mod chest;
mod gold;
mod map;
mod portal;

use bevy::prelude::*;

pub mod prelude {
    pub use super::chest::*;
    pub use super::gold::*;
    pub use super::map::prelude::*;
    pub use super::portal::*;
}

pub fn plugin(app: &mut App) {
    app.add_plugins((chest::plugin, gold::plugin, portal::plugin, map::plugin));
}
