use bevy::prelude::*;

use crate::events::ProjectileHitEvent;
use crate::labels::sets::GamePlaySet;
use crate::status_effects::{
    handle_status_effect_applied, process_status_effects, StatusEffectAppliedEffect,
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
            .add_event::<StatusEffectAppliedEffect>();
    }
}
