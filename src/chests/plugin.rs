use bevy::prelude::*;

use super::{
    handle_open_chest_input::handle_open_chest_input, open_chest::open_chest,
    spawn_chests::spawn_chests,
};

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_chests)
            .add_observer(open_chest)
            .add_observer(handle_open_chest_input);
    }
}
