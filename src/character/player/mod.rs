mod aim;
mod death;
mod input;
mod interact;
mod level;
mod movement;
mod overlay;
mod progression;

pub mod prelude {
    pub use super::aim::PlayerAim;
    pub use super::interact::*;
    pub use super::progression::GameProgress;
    pub use super::{DisplayableStatType, Player, PlayerStats};
}

use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*, ui_widgets::observe};
use bevy_lit::prelude::PointLight2d;
use interact::PlayerInteractionRadius;

use crate::{
    character::{
        Character, Purse, physical_collider,
        player::{
            aim::{AimInput, player_aim},
            input::player_actions,
            movement::PlayerMovement,
        },
    },
    prelude::*,
};

/// How much more experience is required (as a multiplier) after each level up
const PLAYER_LEVEL_REQUIREMENT_MULTIPLIER: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        aim::plugin,
        death::plugin,
        input::plugin,
        interact::plugin,
        level::plugin,
        movement::plugin,
        overlay::plugin,
        progression::plugin,
    ));

    // Player spawn
    app.add_systems(
        OnEnter(AppState::SpawnPlayer),
        (
            spawn_player,
            overlay::spawn_player_overlay,
            transition_to_create_hub,
        )
            .chain(),
    );

    // Player despawn
    app.add_observer(despawn_all::<RestartEvent, Player>);
}

#[derive(Component)]
#[require(
    Character,
    ItemCapacity(50),
    Health::new(100.0),
    SimpleMotion::new(250.0),
    // Double the mass of npcs/enemies so the player can push them around more
    Mass(100.0),
    IFrames,
    Purse
)]
pub struct Player {
    current_level: u32,
    // Outside systems may give the player experience, like when an enemy dies
    pub current_experience: f32,
    next_level_experience_req: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            current_level: 1,
            current_experience: 0.0,
            next_level_experience_req: 20.0,
        }
    }
}

impl Player {
    /// Attempts to increase player level based on current experience and level requirement, and then
    /// sets the new level requirement based on `PLAYER_LEVEL_REQUIREMENT_MULTIPLIER`
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
    sprite_layouts: Res<SpriteSheetLayouts>,
    game_progress: Res<GameProgress>,
    shadows: Res<Shadows>,
    gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    let fire_staff = commands.spawn(fire_staff(&sprites, &sprite_layouts)).id();

    let player = commands
        .spawn((
            Player::default(),
            player_actions(),
            Mana::new(100.0, 10.0),
            game_progress.base_stats.clone(),
            Sprite::from_atlas_image(
                sprites.player_sprite_sheet.clone(),
                TextureAtlas {
                    layout: sprite_layouts.player_atlas_layout.clone(),
                    ..default()
                },
            ),
            related!(Items[
                ice_staff(&sprites, &sprite_layouts),
                sword(&sprites),
                axe(&sprites),
                freeze_axe(&sprites),
                magic_shield(&sprites, &sprite_layouts),
                knight_shield(&sprites, &sprite_layouts),
                health_potion(&sprites),
                tome_of_healing(&sprites)
            ]),
            observe(death::on_player_defeated),
            observe(overlay::on_equipment_use_failed),
            children![
                player_aim(gizmo_assets),
                shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
                physical_collider(),
                hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox),
                (
                    PointLight2d {
                        color: Color::WHITE,
                        intensity: 0.8,
                        outer_radius: 100.0,
                        falloff: 5.0,
                        ..default()
                    },
                    Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0)
                ),
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
        .id();

    commands
        .spawn(fireball(&sprites, &sprite_layouts, 0.0))
        .remove::<(Lifespan, Disabled)>()
        .insert(Transform::from_xyz(0.0, 0.0, 10.0));

    commands.trigger(Equip {
        item: fire_staff,
        holder: player,
    });
}

fn transition_to_create_hub(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::CreateHub);
}
