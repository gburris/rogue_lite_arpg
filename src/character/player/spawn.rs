use avian2d::prelude::{RigidBody, *};
use bevy::prelude::*;

use super::input::{player_actions, player_aim};
use super::interact::PlayerInteractionRadius;
use super::{Player, overlay};
use crate::character::physical_collider;
use crate::prelude::*;

pub(super) fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    game_progress: Res<GameProgress>,
    shadows: Res<Shadows>,
) {
    let fire_staff = commands
        .spawn_scene(fire_staff(&sprites, &sprite_layouts))
        .id();

    let attributes = game_progress.base_attributes;
    let sprite = Sprite::from_atlas_image(
        sprites.player_sprite_sheet.clone(),
        TextureAtlas {
            layout: sprite_layouts.player.clone(),
            ..default()
        },
    );

    let player = commands
        .spawn_scene(bsn! {
            #Player
            Player
            @pause_actions()
            @player_actions()
            ~Mana::new(100.0, 10.0)
            attributes
            template(move |_| Ok(sprite.clone()))
            on(on_player_defeated)
            on(overlay::on_action_failed)
            Items [
                @ice_staff(&sprites, &sprite_layouts)
                --
                @sword(&sprites)
                --
                @axe(&sprites)
                --
                @freeze_axe(&sprites)
                --
                @magic_shield(&sprites, &sprite_layouts)
                --
                @knight_shield(&sprites, &sprite_layouts)
                --
                @health_potion(&sprites)
                --
                @tome_of_healing(&sprites)
            ]
            Children [
                @player_aim()
                --
                @shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0)
                --
                @physical_collider()
                --
                @hurtbox(Vec2::new(24.0, 40.0), GameCollisionLayer::AllyHurtBox)
                --
                @point_light(Color::WHITE, 0.4, 100.0, 5.0)
                Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0)
                --
                PlayerInteractionRadius
                Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0)
                ~CollisionLayers::new(
                    [GameCollisionLayer::PlayerInteractionRadius],
                    [GameCollisionLayer::Interaction],
                )
            ]
        })
        .id();

    commands.trigger(Equip {
        item: fire_staff,
        holder: player,
    });
}

pub(super) fn transition_to_create_hub(mut next_map: NextMap) -> Result {
    next_map.set(MapRequest::Hub)
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        finish_death_animation
            .in_set(VfxSystems::Animate)
            .run_if(in_state(PlayingState::Death)),
    );

    app.insert_resource(GameProgress::default());

    app.add_observer(on_restart);
}

#[derive(Component, Deref, DerefMut)]
struct GameOverTimer(Timer);

fn on_player_defeated(
    _: On<Defeated>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
    mut next_playing_state: ResMut<NextState<PlayingState>>,
) {
    commands
        .entity(*player)
        .insert(GameOverTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        // The colliders go with the children, and a dynamic body without them has no inertia.
        .remove::<(Health, RigidBody)>()
        .despawn_related::<Children>();
    next_playing_state.set(PlayingState::Death);
}

fn finish_death_animation(
    time: Res<Time>,
    player_death_timer_single: Single<&mut GameOverTimer, With<Player>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let mut death_timer = player_death_timer_single.into_inner();
    death_timer.tick(time.delta());
    if death_timer.is_finished() {
        next_app_state.set(AppState::GameOver);
    }
}

#[derive(Resource)]
pub struct GameProgress {
    pub death_counter: u32,
    pub total_career_level: u32,
    pub progress_points: u32,
    pub base_attributes: PlayerAttributes, //Base attributes are upgraded at the NPC each run
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            death_counter: 0,
            total_career_level: 0,
            progress_points: 5,
            base_attributes: PlayerAttributes::default(),
        }
    }
}

/// Triggers when restart is clicked after death in a run.
fn on_restart(
    restart: On<Restart>,
    mut game_progress: ResMut<GameProgress>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    game_progress.death_counter += 1;
    game_progress.total_career_level += restart.player_level;
    game_progress.progress_points += restart.player_level;

    next_app_state.set(AppState::SpawnPlayer);
}
