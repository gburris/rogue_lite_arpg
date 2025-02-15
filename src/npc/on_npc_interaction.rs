use avian2d::prelude::CollidingEntities;

use bevy::prelude::*;

use crate::{
    labels::states::PausedState,
    player::{systems::PauseInputEvent, AttemptInteractionInput, Player},
};

use super::{components::NPCInteractionRadius, events::NPCInteraction};

pub fn on_npc_input_interaction(
    _: Trigger<AttemptInteractionInput>,
    mut commands: Commands,
    query: Query<(&Parent, &CollidingEntities), With<NPCInteractionRadius>>,
    player_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_query.into_inner();
    for (parent, colliding_entities) in &query {
        if colliding_entities.contains(&player_entity) {
            commands.trigger_targets(NPCInteraction, parent.get());
        }
    }
}

pub fn on_shop_keeper_store_open(_: Trigger<NPCInteraction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::Inventory),
    });
}

pub fn on_stat_trainer_store_open(_: Trigger<NPCInteraction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::StatsShop),
    });
}

pub fn on_game_guide_start(_: Trigger<NPCInteraction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::Inventory),
    });
}
