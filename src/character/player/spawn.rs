use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::ui_widgets::observe;
use bevy_lit::prelude::PointLight2d;

use super::aim::player_aim;
use super::input::player_actions;
use super::interact::PlayerInteractionRadius;
use super::{Player, death, overlay};
use crate::character::physical_collider;
use crate::character::vision::TargetInfo;
use crate::prelude::*;

pub(super) fn spawn_player(
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
            PauseControl,
            pause_actions(),
            player_actions(),
            Mana::new(100.0, 10.0),
            game_progress.base_stats,
            Sprite::from_atlas_image(
                sprites.player_sprite_sheet.clone(),
                TextureAtlas {
                    layout: sprite_layouts.player.clone(),
                    ..default()
                },
            ),
            TargetInfo::default(),
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
            observe(overlay::on_equipment_use_succeeded),
            observe(overlay::on_equipment_use_failed),
            children![
                player_aim(gizmo_assets),
                shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
                physical_collider(),
                hurtbox(Vec2::new(24.0, 40.0), GameCollisionLayer::AllyHurtBox),
                (
                    PointLight2d {
                        color: Color::WHITE,
                        intensity: 0.4,
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

    commands.trigger(Equip {
        item: fire_staff,
        holder: player,
    });
}

pub(super) fn transition_to_create_hub(mut next_map: NextMap) -> Result {
    next_map.set(MapRequest::Hub)
}
