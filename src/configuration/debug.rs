use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{GREEN, RED, YELLOW},
    dev_tools::states::log_transitions,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    log::{Level, LogPlugin},
    prelude::*,
};

#[cfg(not(target_arch = "wasm32"))]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

use crate::prelude::{
    AppState, AttackState, Brain, Enemy, EnemyKind, InGameSystems, MapBlueprint, Menu, MotionMode,
    Placement, PlacementKind, TargetInfo, Targeting, TileKind, TileRect, Vision,
    ZLayer,
};

use super::view;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::INFO,
                // update game dev project to info or another when you get tired of debug
                filter: "wgpu=error,baba_yaga=debug".to_string(),
                ..default()
            })
            .set(view::get_window_plugin())
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(PhysicsDebugPlugin)
    .insert_gizmo_config(
        PhysicsGizmos::default(),
        GizmoConfig {
            enabled: false,
            ..default()
        },
    )
    // Enable system ambiguity detection
    .edit_schedule(Update, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Warn,
            ..default()
        });
    })
    .add_systems(
        Update,
        (
            handle_debug_input
                .in_set(InGameSystems::PlayerInput)
                .ambiguous_with_all(),
            (
                view::camera_debug_system,
                debug_vision,
                spawn_enemy_ai_labels,
                update_enemy_ai_labels,
                debug_target_lines,
            )
                .in_set(InGameSystems::HudOverlay)
                .run_if(resource_exists::<DebugRenderEnabled>),
            cleanup_enemy_ai_labels
                .in_set(InGameSystems::HudOverlay)
                .run_if(not(resource_exists::<DebugRenderEnabled>)),
        ),
    );

    app.add_systems(Update, log_transitions::<AppState>);
    app.add_systems(Update, log_transitions::<Menu>);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(FpsOverlayPlugin::default());
}

#[derive(Resource)]
struct DebugRenderEnabled;

fn handle_debug_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
    debug_enabled: Option<Res<DebugRenderEnabled>>,
    app_state: Res<State<AppState>>,
    map_blueprints: Query<Entity, With<MapBlueprint>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Comma) {
        if debug_enabled.is_some() {
            commands.remove_resource::<DebugRenderEnabled>();
        } else {
            commands.insert_resource(DebugRenderEnabled);
        }
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
    // Teleports into a small walkable arena of 15x15 rooms with hardcoded
    // enemy combinations for AI behavior testing. The exit portal returns to
    // normal map generation.
    if keyboard_input.just_pressed(KeyCode::F1) && *app_state == AppState::Playing {
        for old_map in &map_blueprints {
            commands.entity(old_map).despawn();
        }
        commands.spawn(build_test_arena_blueprint());
        next_state.set(AppState::SpawnMap);
        info!("Spawning AI test arena: lobby plus 3x2 rooms of 15x15 tiles (F1)");
    }
}

/// Draws debug gizmos for AI vision direction and cone angles.
fn debug_vision(mut gizmos: Gizmos, query: Query<(&Transform, &Vision)>) {
    for (transform, vision) in &query {
        let origin = transform.translation.xy();
        let forward = vision.aim_direction;

        gizmos.arrow_2d(origin, origin + forward * 64.0, GREEN);

        let left = forward.rotate(Vec2::from_angle(45f32.to_radians()));
        let right = forward.rotate(Vec2::from_angle(-45f32.to_radians()));

        gizmos.line_2d(origin, origin + left * 64.0, YELLOW);
        gizmos.line_2d(origin, origin + right * 64.0, YELLOW);
    }
}

/// Draws a line from each targeting enemy toward its current target position.
/// Green means a clear ray, red means the ray is blocked.
fn debug_target_lines(
    mut gizmos: Gizmos,
    enemies: Query<(&Transform, &TargetInfo, Has<Targeting>), With<Enemy>>,
) {
    for (transform, target_info, has_target) in &enemies {
        if !has_target {
            continue;
        }
        let origin = transform.translation.xy();
        let color = if target_info.line_of_sight {
            GREEN
        } else {
            RED
        };
        gizmos.line_2d(
            origin,
            origin + target_info.direction * target_info.distance,
            color,
        );
    }
}

/// Marker on the floating [`Text2d`] above a debug-tracked enemy.
#[derive(Component)]
struct EnemyAiLabel;

/// Marker on enemies that already have a label child, so spawn runs once.
#[derive(Component)]
struct HasAiLabel;

/// Height of the floating label above the enemy's center, in world units.
const AI_LABEL_HEIGHT: f32 = 30.0;

/// Spawns one floating label per enemy while the debug overlay is enabled.
fn spawn_enemy_ai_labels(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<HasAiLabel>)>,
) {
    for enemy in &enemies {
        commands.entity(enemy).insert(HasAiLabel);
        commands.entity(enemy).with_child((
            EnemyAiLabel,
            Text2d::new("..."),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor::from(Color::WHITE),
            Transform::from_translation(Vec3::new(
                0.0,
                AI_LABEL_HEIGHT,
                ZLayer::AboveSprite.z(),
            )),
        ));
    }
}

/// Refreshes each label in plain words: the motion mode, the target distance
/// in world units, and whether the enemy sees you and is attacking.
fn update_enemy_ai_labels(
    enemies: Query<
        (
            &Brain,
            &TargetInfo,
            Has<Targeting>,
            Option<&AttackState>,
        ),
        With<Enemy>,
    >,
    mut labels: Query<(&ChildOf, &mut Text2d), With<EnemyAiLabel>>,
) {
    for (parent, mut text) in &mut labels {
        let Ok((motion, target_info, has_target, attack_state)) = enemies.get(parent.parent())
        else {
            continue;
        };
        let attacking = attack_state.is_some_and(|attack| attack.is_attacking);
        let mode = short_activity(motion);
        if has_target {
            let sight = if target_info.line_of_sight {
                "sees you"
            } else {
                "blocked"
            };
            text.0 = if attacking {
                format!(
                    "{mode}\ntarget {:.0}\n{sight}, attacking",
                    target_info.distance
                )
            } else {
                format!("{mode}\ntarget {:.0}\n{sight}", target_info.distance)
            };
        } else if attacking {
            text.0 = format!("{mode}\nattacking");
        } else {
            text.0 = mode.to_string();
        }
    }
}

/// Removes floating labels once the debug overlay is toggled off.
fn cleanup_enemy_ai_labels(
    mut commands: Commands,
    labels: Query<Entity, With<EnemyAiLabel>>,
    marked: Query<Entity, With<HasAiLabel>>,
) {
    for label in &labels {
        commands.entity(label).despawn();
    }
    for enemy in &marked {
        commands.entity(enemy).remove::<HasAiLabel>();
    }
}

/// One-word name for what the enemy is doing: its maneuver while one runs,
/// otherwise its locomotion mode.
fn short_activity(brain: &Brain) -> &'static str {
    match brain.maneuver() {
        Some(maneuver) => maneuver.name(),
        None => short_mode(brain.mode()),
    }
}

/// One-word name for the enemy's current motion mode.
fn short_mode(mode: MotionMode) -> &'static str {
    match mode {
        MotionMode::Idle => "Idle",
        MotionMode::Chase => "Chase",
        MotionMode::ReturnToAnchor => "Return",
        MotionMode::Flee => "Flee",
        MotionMode::KeepDistance(_) => "Keep",
        MotionMode::Hold => "Hold",
        MotionMode::Strafe => "Strafe",
        MotionMode::Wander => "Wander",
    }
}

// ---------------------------------------------------------------------------
// AI test arena: enemy-free lobby plus hallway plus 15x15 rooms.
// ---------------------------------------------------------------------------

/// Interior width and height of one arena room, in tiles (15 scaled up 50%,
/// floored to keep tiles integral).
const ARENA_ROOM_TILES: i32 = 22;
/// Room grid dimensions of the arena.
const ARENA_COLS: i32 = 3;
const ARENA_ROWS: i32 = 2;
/// Height of the central hallway separating the room rows, in tiles.
const ARENA_HALLWAY_TILES: i32 = 3;
/// Width of the enemy-free spawn lobby west of the rooms, in tiles.
const ARENA_LOBBY_TILES: i32 = 7;
/// West edge of the room grid: lobby plus its dividing wall.
const ARENA_GRID_WEST: i32 = 1 + ARENA_LOBBY_TILES + 1;

/// Builds a lobby plus a 3x2 arena of 15x15 rooms off one central hallway.
/// Each room has a single centered 2-wide door; walls otherwise block sight
/// lines, so walking the hallway never opens a ray into another room. Ground
/// enemies sit pinned to their back wall outside their 350-unit sight range
/// from the door; fliers stay in the far east rooms. The exit portal returns
/// to normal generation.
fn build_test_arena_blueprint() -> MapBlueprint {
    debug_assert_eq!(
        ARENA_ROWS, 2,
        "arena builder lays one hallway between two rows"
    );
    let map_size = UVec2::new(
        (ARENA_GRID_WEST + ARENA_COLS * ARENA_ROOM_TILES + (ARENA_COLS - 1) + 1) as u32,
        (1 + ARENA_ROOM_TILES + 1 + ARENA_HALLWAY_TILES + 1 + ARENA_ROOM_TILES + 1) as u32,
    );
    let mut blueprint = MapBlueprint::new(map_size, TileKind::Wall, 0);
    let hallway_min_y = 1 + ARENA_ROOM_TILES + 1;

    // Room interiors.
    for room_x in 0..ARENA_COLS {
        for room_y in 0..ARENA_ROWS {
            let min = room_min(room_x, room_y);
            if let Some(rect) = TileRect::new(min, UVec2::splat(ARENA_ROOM_TILES as u32)) {
                blueprint.tiles.fill_rect(&rect, TileKind::Ground);
            }
        }
    }

    // Central hallway spanning the room grid.
    if let Some(rect) = TileRect::new(
        IVec2::new(ARENA_GRID_WEST, hallway_min_y),
        UVec2::new(
            map_size.x - ARENA_GRID_WEST as u32 - 1,
            ARENA_HALLWAY_TILES as u32,
        ),
    ) {
        blueprint.tiles.fill_rect(&rect, TileKind::Ground);
    }

    // Enemy-free lobby opening straight into the hallway's west end.
    if let Some(rect) = TileRect::new(
        IVec2::new(1, hallway_min_y - 2),
        UVec2::new(
            ARENA_LOBBY_TILES as u32,
            (ARENA_HALLWAY_TILES + 4) as u32,
        ),
    ) {
        blueprint.tiles.fill_rect(&rect, TileKind::Ground);
    }
    for y in hallway_min_y..(hallway_min_y + ARENA_HALLWAY_TILES) {
        blueprint
            .tiles
            .set(IVec2::new(ARENA_GRID_WEST - 1, y), TileKind::Ground);
    }

    // One centered 2-wide door per room.
    for room_x in 0..ARENA_COLS {
        for room_y in 0..ARENA_ROWS {
            let min = room_min(room_x, room_y);
            let center = room_center(room_x, room_y);
            let door_y = if room_y == 0 {
                min.y + ARENA_ROOM_TILES
            } else {
                min.y - 1
            };
            for dx in -1..=0 {
                blueprint
                    .tiles
                    .set(IVec2::new(center.x + dx, door_y), TileKind::Ground);
            }
        }
    }

    // Player spawns in the empty lobby; the exit sits in the last room.
    blueprint.placements.push(Placement {
        tile: IVec2::new(
            1 + ARENA_LOBBY_TILES / 2,
            hallway_min_y + ARENA_HALLWAY_TILES / 2,
        ),
        kind: PlacementKind::PlayerStart,
    });
    blueprint.placements.push(Placement {
        tile: room_center(2, 1) + IVec2::new(5, 4),
        kind: PlacementKind::Exit,
    });

    // Hardcoded combinations pinned to the back wall, far from each door.
    place_enemies(
        &mut blueprint,
        0,
        0,
        &[EnemyKind::Warrior, EnemyKind::Warrior],
        -1,
    );
    place_enemies(
        &mut blueprint,
        1,
        0,
        &[EnemyKind::FireMage, EnemyKind::FireMage],
        -1,
    );
    place_enemies(
        &mut blueprint,
        2,
        0,
        &[EnemyKind::IceMage, EnemyKind::IceMage],
        -1,
    );
    place_enemies(
        &mut blueprint,
        0,
        1,
        &[
            EnemyKind::Warrior,
            EnemyKind::FireMage,
            EnemyKind::IceMage,
        ],
        1,
    );
    place_enemies(
        &mut blueprint,
        1,
        1,
        &[EnemyKind::Bat, EnemyKind::Bat, EnemyKind::Bat],
        1,
    );
    place_enemies(
        &mut blueprint,
        2,
        1,
        &[EnemyKind::Warrior, EnemyKind::Bat, EnemyKind::Bat],
        1,
    );

    blueprint
}

/// Bottom-left tile of a room's interior. Row 1 sits above the hallway band.
fn room_min(room_x: i32, room_y: i32) -> IVec2 {
    IVec2::new(
        ARENA_GRID_WEST + room_x * (ARENA_ROOM_TILES + 1),
        1 + room_y * (ARENA_ROOM_TILES + 1) + room_y * (ARENA_HALLWAY_TILES + 1),
    )
}

/// Center tile of a room's interior.
fn room_center(room_x: i32, room_y: i32) -> IVec2 {
    room_min(room_x, room_y) + IVec2::splat(ARENA_ROOM_TILES / 2)
}

/// Pins enemies to the back wall of a room (`back`: -1 for the south row,
/// +1 for the north row) so each door opens onto empty floor.
fn place_enemies(
    blueprint: &mut MapBlueprint,
    room_x: i32,
    room_y: i32,
    kinds: &[EnemyKind],
    back: i32,
) {
    let center = room_center(room_x, room_y);
    let offsets = [
        IVec2::new(-4, 5 * back),
        IVec2::new(4, 5 * back),
        IVec2::new(0, 4 * back),
    ];
    for (index, kind) in kinds.iter().enumerate() {
        blueprint.placements.push(Placement {
            tile: center + offsets[index % offsets.len()],
            kind: PlacementKind::Enemy(*kind),
        });
    }
}
