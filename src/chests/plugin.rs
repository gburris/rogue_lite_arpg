use bevy::prelude::*;

use crate::chests::{spawn_chests::*, *};

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_chests)
            .add_observer(open_chest::open_chest)
            .add_observer(open_chest::on_open_chest_input);
    }
}
