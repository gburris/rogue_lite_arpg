use bevy::prelude::*;

use crate::{combat::damage::*, labels::sets::InGameSet};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            invulnerable::handle_invulnerability.in_set(InGameSet::Simulation),
        )
        .add_observer(damage::on_damage_event);
    }
}
