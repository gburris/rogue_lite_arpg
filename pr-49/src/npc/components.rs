use bevy::prelude::*;

use crate::{
    character::Character,
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{spawn_axe, spawn_ice_staff, spawn_sword},
    player::interact::InteractionEvent,
};

use super::{on_game_guide_start, on_shop_keeper_store_open, on_stat_trainer_store_open};

#[derive(Component)]
#[require(Character)]
pub struct NPC;

#[derive(Debug, Clone, Component, Copy)]
pub enum NPCType {
    Helper,
    Shopkeeper,
    StatTrainer,
}

impl NPCType {
    pub fn spawn_weapon(
        &self,
        commands: &mut Commands,
        sprites: &SpriteAssets,
        atlases: &SpriteSheetLayouts,
    ) -> Entity {
        match self {
            NPCType::Helper => spawn_ice_staff(commands, sprites, atlases),
            NPCType::Shopkeeper => spawn_axe(commands, sprites),
            NPCType::StatTrainer => spawn_sword(commands, sprites),
        }
    }

    pub fn get_sprite_sheet(&self, sprites: &SpriteAssets) -> Handle<Image> {
        match self {
            NPCType::Helper => sprites.game_guide_sprite_sheet.clone(),
            NPCType::Shopkeeper => sprites.shop_keeper_sprite_sheet.clone(),
            NPCType::StatTrainer => sprites.stat_trainer_sprite_sheet.clone(),
        }
    }

    pub fn get_interaction_observer(&self) -> fn(Trigger<InteractionEvent>, Commands) {
        match self {
            NPCType::Helper => on_game_guide_start,
            NPCType::Shopkeeper => on_shop_keeper_store_open,
            NPCType::StatTrainer => on_stat_trainer_store_open,
        }
    }
}
