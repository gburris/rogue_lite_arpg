use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{
        attributes::{mana::Mana, Health},
        damage::components::HasIFrames,
        weapon::staffs::spawn_fire_staff,
    },
    configuration::{assets::SpriteAssets, GameCollisionLayer},
    items::{spawn_health_potion, spawn_helmet, spawn_shovel, spawn_sword},
    labels::{layer::ZLayer, states::AppState},
    movement::components::SimpleMotion,
    player::{systems::*, Inventory, Player, PlayerEquipmentSlots, PlayerStats},
};

#[derive(Component, Default)]
pub struct AimPosition {
    pub position: Vec2, // position where entitiy is aiming, for player this is the cursor
}

pub fn player_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    sprites: Res<SpriteAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    //Player Inventory Setup
    let mut inventory = Inventory::default_inventory();
    let _ = inventory.add_item(spawn_health_potion(&mut commands));
    let _ = inventory.add_item(spawn_sword(&mut commands, &sprites));
    let _ = inventory.add_item(spawn_helmet(&mut commands, &sprites));
    let _ = inventory.add_item(spawn_shovel(&mut commands, &sprites));

    inventory
        .add_item(spawn_fire_staff(&mut commands, &sprites))
        .ok();

    //Player Sprite Sheet Setup
    //TODO: Add all atlas indecies values to a config/map
    // Move all this somewhere else
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 13, 21, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    //Idle is on row 8
    //Facing up has 4 sprites in it's row
    let idle_down_animation_indices = AnimationIndices {
        first: 20 * 13,        // 260
        last: 20 * 13 + 2 - 1, // 263
    };

    commands
        .spawn((
            Player,
            PlayerStats::default(),
            AimPosition::default(),
            SimpleMotion::new(450.0),
            Health::new(100.0),
            Mana::new(100.0, 10.0),
            inventory,
            PlayerEquipmentSlots::default(),
            HasIFrames {
                duration: Duration::from_secs(1),
            },
            RigidBody::Dynamic,
            Collider::rectangle(100.0, 100.0),
            CollisionLayers::new(
                [GameCollisionLayer::Player, GameCollisionLayer::Grounded],
                [
                    GameCollisionLayer::Enemy,
                    GameCollisionLayer::Interaction,
                    GameCollisionLayer::InAir,
                    GameCollisionLayer::Grounded,
                    GameCollisionLayer::HighObstacle,
                    GameCollisionLayer::LowObstacle,
                ],
            ),
            LockedAxes::new().lock_rotation(),
            (
                AnimationTimer(Timer::from_seconds(2.0, TimerMode::Repeating)), // <-- And this
                Sprite::from_atlas_image(
                    sprites.player_sprite_sheet.clone(),
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: idle_down_animation_indices.first,
                    },
                ),
                idle_down_animation_indices,
                MovementDirection::None,
            ),
            Transform::from_xyz(0., 0., ZLayer::Player.z()),
        ))
        .observe(death::on_player_defeated)
        .observe(equip::on_main_hand_activated);
    game_state.set(AppState::CreateHub);
}
