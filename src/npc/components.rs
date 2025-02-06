use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::attributes::Health,
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer,
    },
    items::{spawn_axe, spawn_ice_staff, spawn_sword},
    movement::components::SimpleMotion,
};

use super::{
    events::NPCInteraction, on_game_guide_start, on_shop_keeper_store_open,
    on_stat_trainer_store_open,
};

#[derive(Component)]
#[require(
    Health,
    SimpleMotion,
    Collider(|| Collider::rectangle(32.0, 32.0)),
    RigidBody(|| RigidBody::Kinematic),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Grounded, [GameCollisionLayer::Grounded, GameCollisionLayer::InAir]))
)]
pub struct NPC;

#[derive(Component)]
#[require(
    CollidingEntities,
    Sensor,
    Collider(||  Collider::circle(70.0)),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::Interaction, [GameCollisionLayer::Player]))

)]
pub struct NPCInteractionRadius;

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
            NPCType::Helper => spawn_ice_staff(commands, &sprites, &atlases),
            NPCType::Shopkeeper => spawn_axe(commands, &sprites),
            NPCType::StatTrainer => spawn_sword(commands, &sprites),
        }
    }

    pub fn get_sprite_sheet(&self, sprites: &SpriteAssets) -> Handle<Image> {
        match self {
            NPCType::Helper => sprites.game_guide_sprite_sheet.clone(),
            NPCType::Shopkeeper => sprites.shop_keeper_sprite_sheet.clone(),
            NPCType::StatTrainer => sprites.stat_trainer_sprite_sheet.clone(),
        }
    }

    pub fn get_observer(&self) -> fn(Trigger<NPCInteraction>, Commands) {
        match self {
            NPCType::Helper => on_game_guide_start,
            NPCType::Shopkeeper => on_shop_keeper_store_open,
            NPCType::StatTrainer => on_stat_trainer_store_open,
        }
    }
}
