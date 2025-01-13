use bevy::prelude::*;

use crate::{combat::damage::systems::*, labels::sets::GamePlaySet};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_invulnerability.in_set(GamePlaySet::Simulation),
        )
        .add_observer(on_damage_event);
    }
}
