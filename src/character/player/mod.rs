use avian2d::prelude::*;
use bevy::prelude::*;
use movement::PlayerMovementEvent;

mod aim;
mod death;
mod input;
pub mod interact;
mod level;
mod movement;

pub use input::PauseInputEvent;

use crate::{
    character::{physical_collider, player::interact::PlayerInteractionRadius, Character},
    combat::{damage::hurtbox, invulnerable::IFrames, Health, Mana},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
    },
    items::{
        self,
        equipment::{on_equipment_activated, on_equipment_deactivated, Equipped},
        inventory::Inventory,
    },
    labels::{
        sets::InGameSet,
        states::{AppState, PlayingState},
    },
    map::systems::state::transition_to_create_hub,
    prelude::*,
    progression::GameProgress,
};

/// How much more experience is required (as a multiplier) after each level up
const PLAYER_LEVEL_REQUIREMENT_MULTIPLIER: f32 = 2.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_systems(
                OnEnter(AppState::SpawnPlayer),
                (spawn_player, transition_to_create_hub).chain(),
            )
            .add_systems(
                Update,
                death::finish_death_animation
                    .in_set(InGameSet::Vfx)
                    .run_if(in_state(PlayingState::Death)),
            )
            .add_systems(
                Update,
                input::player_input
                    .in_set(InGameSet::PlayerInput)
                    .run_if(in_state(PlayingState::Playing)),
            )
            .add_systems(
                Update,
                (
                    (
                        movement::player_movement,
                        aim::update_player_aim,
                        level::on_player_experience_change,
                    )
                        .in_set(InGameSet::Simulation),
                    (aim::draw_cursor, level::animate_level_up).in_set(InGameSet::Vfx),
                ),
            )
            .add_observer(level::on_level_up)
            .add_observer(movement::on_player_stopped)
            .add_observer(interact::on_player_interaction_input)
            .add_observer(interact::on_interaction_zone_added);
    }
}

#[derive(Component)]
#[require(
    Character,
    Health::new(100.0),
    SimpleMotion::new(250.0),
    // Double the mass of npcs/enemies so the player can push them around more
    Mass(100.0),
    IFrames,
)]
pub struct Player {
    pub aim_position: Vec2, // tracks the cursor
    current_level: u32,
    // Outside systems may give the player experience, like when an enemy dies
    pub current_experience: f32,
    next_level_experience_req: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            aim_position: Vec2::ZERO,
            current_level: 1,
            current_experience: 0.0,
            next_level_experience_req: 20.0,
        }
    }
}

impl Player {
    /// Attempts to increase player level based on current experience and level requirement, and then
    /// sets the new level requirement based on PLAYER_LEVEL_REQUIREMENT_MULTIPLIER
    ///
    /// returns whether the player leveled up
    pub fn attempt_level_up(&mut self) -> bool {
        if self.current_experience >= self.next_level_experience_req {
            self.current_experience -= self.next_level_experience_req;
            self.next_level_experience_req *= PLAYER_LEVEL_REQUIREMENT_MULTIPLIER;
            self.current_level += 1;
            return true;
        }

        false
    }

    pub fn get_progress_to_next_level(&self) -> f32 {
        self.current_experience / self.next_level_experience_req
    }

    pub fn get_level(&self) -> u32 {
        self.current_level
    }
}

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub agility: u32,   //Movement speed, roll range
    pub strength: u32,  //Melee swing damage
    pub dexterity: u32, //Critical Stike Change
    pub intellect: u32, //Spell damage
    pub luck: u32,      //Drop rate
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            agility: 1,
            strength: 1,
            dexterity: 1,
            intellect: 1,
            luck: 99,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum DisplayableStatType {
    Agility,
    Strength,
    Dexterity,
    Intellect,
    Luck,
}

impl DisplayableStatType {
    pub fn get_description(&self) -> &'static str {
        match self {
            DisplayableStatType::Agility => "Movement speed, roll range",
            DisplayableStatType::Strength => "Melee swing damage",
            DisplayableStatType::Dexterity => "Critical Strike Chance",
            DisplayableStatType::Intellect => "Spell damage",
            DisplayableStatType::Luck => "Drop rate",
        }
    }

    pub fn get_value(&self, stats: &PlayerStats) -> u32 {
        match self {
            DisplayableStatType::Agility => stats.agility,
            DisplayableStatType::Strength => stats.strength,
            DisplayableStatType::Dexterity => stats.dexterity,
            DisplayableStatType::Intellect => stats.intellect,
            DisplayableStatType::Luck => stats.luck,
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    texture_layouts: Res<SpriteSheetLayouts>,
    game_progress: Res<GameProgress>,
    atlases: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
) {
    let starting_items = [
        items::spawn_ice_staff(&mut commands, &sprites, &texture_layouts),
        items::spawn_health_potion(&mut commands, &sprites),
        items::spawn_sword(&mut commands, &sprites),
        items::spawn_offhand(&mut commands, &sprites, &texture_layouts, "tome_of_healing"),
        items::spawn_offhand(&mut commands, &sprites, &texture_layouts, "magic_shield"),
        items::spawn_offhand(&mut commands, &sprites, &texture_layouts, "knight_shield"),
    ];

    let player = commands
        .spawn((
            Player::default(),
            Inventory::builder()
                .items(starting_items.into())
                .coins(0)
                .max_capacity(50)
                .build(),
            Mana::new(100.0, 10.0),
            game_progress.base_stats.clone(),
            Sprite::from_atlas_image(
                sprites.player_sprite_sheet.clone(),
                TextureAtlas {
                    layout: atlases.player_atlas_layout.clone(),
                    ..default()
                },
            ),
            children![
                shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
                physical_collider(),
                hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox),
                (
                    PlayerInteractionRadius,
                    Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
                    CollisionLayers::new(
                        [GameCollisionLayer::PlayerInteractionRadius],
                        [GameCollisionLayer::Interaction],
                    ),
                )
            ],
        ))
        .add_children(&starting_items)
        .observe(death::on_player_defeated)
        .observe(on_equipment_activated)
        .observe(on_equipment_deactivated)
        .id();

    commands
        .entity(starting_items[0])
        .insert(Equipped::new(player));

    info!("Player spawned: {}", player);
}
