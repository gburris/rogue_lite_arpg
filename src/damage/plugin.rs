use bevy::prelude::*;

use crate::damage::systems::handle_damage;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_damage);
    }
}
