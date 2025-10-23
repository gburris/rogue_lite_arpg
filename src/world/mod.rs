pub mod chest;
pub mod gold;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((chest::plugin, gold::plugin));
}
