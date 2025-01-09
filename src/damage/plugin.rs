use bevy::prelude::*;

use super::handle_damage;
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_damage);
    }
}
