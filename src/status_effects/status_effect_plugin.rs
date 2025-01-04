use bevy::prelude::*;

use crate::{
    labels::sets::GamePlaySet,
    projectile::events::ProjectileHitEvent,
    status_effects::{
        events::StatusEffectAppliedEvent,
        {handle_status_effect_applied, process_status_effects},
    },
};

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>()
            .add_systems(
                Update,
                (handle_status_effect_applied, process_status_effects)
                    .in_set(GamePlaySet::Simulation),
            )
            .add_event::<StatusEffectAppliedEvent>();
    }
}
