use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    ai::{state::ActionState, SimpleMotion},
    animation::AnimationTimer,
    combat::Health,
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        YSort,
    },
    items::{spawn_axe, spawn_ice_staff, spawn_sword},
    player::interact::InteractionEvent,
};

use super::{on_game_guide_start, on_shop_keeper_store_open, on_stat_trainer_store_open};

#[derive(Component)]
#[require(
    Health,
    SimpleMotion,
    RigidBody(|| RigidBody::Kinematic),
    LockedAxes(|| LockedAxes::new().lock_rotation()),
    ActionState,
    AnimationTimer,
    YSort
)]
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
        sprites: &Res<SpriteAssets>,
        atlases: &Res<SpriteSheetLayouts>,
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
