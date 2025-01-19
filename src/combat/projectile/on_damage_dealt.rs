use bevy::prelude::*;

use crate::combat::damage::events::DealtDamageEvent;

// For certain entities, like projectiles, they have no concept of "health" but instead despawn after "X" hits
pub fn on_damage_dealt_despawn(trigger: Trigger<DealtDamageEvent>, mut commands: Commands) {
    commands.entity(trigger.entity()).despawn_recursive();
}
