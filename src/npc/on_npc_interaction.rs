use avian2d::prelude::CollidingEntities;

use bevy::prelude::*;

use crate::{
    labels::states::PausedState,
    npc::events::{GameGuideStart, StatTrainerStoreOpen},
    player::{systems::PauseInputEvent, AttemptInteractionInput, Player},
};

use super::{
    components::NPCInteractionRadius,
    events::{NPCInteraction, ShopKeeperStoreOpen},
    setup::NPCType,
    NPC,
};

pub fn on_npc_input_interaction(
    _: Trigger<AttemptInteractionInput>,
    mut commands: Commands,
    query: Query<(&Parent, &CollidingEntities), With<NPCInteractionRadius>>,
    player_query: Query<Entity, With<Player>>,
) {
    warn!("player pressed space");
    let player_entity = player_query.single();
    for (parent, colliding_entities) in &query {
        if colliding_entities.contains(&player_entity) {
            commands.trigger_targets(NPCInteraction, parent.get());
        }
    }
}

pub fn dispatch_npc_interaction(
    npc_interaction: Trigger<NPCInteraction>,
    mut commands: Commands,
    query: Query<(&NPC, &NPCType)>,
) {
    if let Ok((_npc, npc_type)) = query.get(npc_interaction.entity()) {
        match npc_type {
            NPCType::Shopkeeper => commands.trigger(ShopKeeperStoreOpen),
            NPCType::StatTrainer => commands.trigger(StatTrainerStoreOpen),
            NPCType::Helper => commands.trigger_targets(GameGuideStart, npc_interaction.entity()),
        }
    }
}

pub fn on_shop_keeper_store_open(_: Trigger<ShopKeeperStoreOpen>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::Inventory),
    });
}

pub fn on_stat_trainer_store_open(_: Trigger<StatTrainerStoreOpen>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::StatsShop),
    });
    warn!("stat trainer interaction on npc");
}

pub fn on_game_guide_start(_: Trigger<GameGuideStart>, mut commands: Commands) {
    warn!("on_game_guide_start");
}
