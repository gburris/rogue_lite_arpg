use avian2d::prelude::LinearVelocity;
use bevy::camera::NormalizedRenderTarget;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{BrpError, BrpResult, RemotePlugin};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{PrimaryWindow, WindowRef};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::prelude::*;

/// Most hits kept between two snapshots; older ones are dropped first.
const MAX_RECORDED_HITS: usize = 512;
/// Frames a tapped key stays down, so actions that read held keys see it.
const TAP_FRAMES: u32 = 2;

/// Serves the Bevy Remote Protocol on `127.0.0.1:15702`, adding `game/…` methods to the built-in
/// `world.…` ones so a script can play the game and read it back. Each handler's doc lists its
/// params, e.g. `curl -d '{"jsonrpc":"2.0","id":1,"method":"game/snapshot"}' 127.0.0.1:15702`.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        RemotePlugin::default()
            .with_method_main("game/start", start_game)
            .with_method_main("game/load_map", load_map)
            .with_method_main("game/snapshot", take_snapshot)
            .with_method_main("game/ui", list_ui_text)
            .with_method_main("game/player_input", set_player_input)
            .with_method_main("game/key", press_key)
            .with_method_main("game/click", click)
            .with_method_main("game/teleport_player", teleport_player)
            .with_method_main("game/spawn_enemies", spawn_enemies)
            .with_method_main("game/set_player_health", set_player_health)
            .with_method_main("game/set_invincible", set_invincible)
            .with_method_main("game/time", control_time)
            .with_method_main("game/screenshot", take_screenshot)
            .with_method_main("game/exit", exit_game),
        RemoteHttpPlugin::default(),
    ));
    app.init_resource::<RecordedHits>()
        .init_resource::<PendingKeyReleases>()
        .init_resource::<PendingClicks>()
        .add_observer(on_damage_dealt_record_hit)
        .add_systems(
            Update,
            apply_scripted_player_input
                .in_set(InGameSystems::PlayerInput)
                .run_if(resource_exists::<ScriptedPlayerInput>),
        )
        // In `Last`, so they run in menus too. Remote methods run after `Last`, so these see a
        // request on the next frame.
        .add_systems(Last, (release_tapped_keys, press_pending_clicks));
}

/// Input a script holds on the player, applied every frame until replaced or released.
#[derive(Resource, Clone, Copy, Debug, Default, Deserialize)]
#[serde(default)]
struct ScriptedPlayerInput {
    #[serde(rename = "move")]
    movement: Option<Vec2>,
    aim: Option<Vec2>,
    mainhand: bool,
    offhand: bool,
    release: bool,
}

/// Hits landed since the last `game/snapshot`.
#[derive(Resource, Debug, Default, Deref, DerefMut)]
struct RecordedHits(Vec<Value>);

/// Keys tapped with `game/key`, released once their frames run out.
#[derive(Resource, Debug, Default, Deref, DerefMut)]
struct PendingKeyReleases(Vec<PendingKeyRelease>);

/// A tapped key and the frames it stays down.
#[derive(Clone, Copy, Debug)]
struct PendingKeyRelease {
    key: KeyCode,
    frames_left: u32,
}

fn on_damage_dealt_record_hit(
    damage_dealt: On<DamageDealt>,
    time: Res<Time<Virtual>>,
    mut recorded_hits: ResMut<RecordedHits>,
) {
    if recorded_hits.len() >= MAX_RECORDED_HITS {
        recorded_hits.remove(0);
    }
    recorded_hits.push(json!({
        "time": time.elapsed_secs(),
        "entity": damage_dealt.entity.to_bits(),
        "damage": damage_dealt.damage,
        "attacker": damage_dealt.attacker.map(Entity::to_bits),
    }));
}

fn parse_params<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> BrpResult<T> {
    serde_json::from_value(params.unwrap_or(Value::Null))
        .map_err(|error| BrpError::internal(format!("bad params: {error}")))
}

/// `game/start`: leaves the start screen, as its button does.
#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
fn start_game(
    In(_): In<Option<Value>>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) -> BrpResult {
    if *app_state.get() == AppState::StartScreen {
        next_app_state.set(AppState::AssetLoading);
    }
    Ok(json!({ "state": format!("{:?}", app_state.get()) }))
}

#[derive(Deserialize)]
struct LoadMapParams {
    recipe: String,
    seed: Option<u64>,
}

/// `game/load_map {recipe, seed?}`: builds a map recipe by name, replacing the current map.
fn load_map(In(params): In<Option<Value>>, mut next_map: NextMap) -> BrpResult {
    let LoadMapParams { recipe, seed } = parse_params(params)?;
    next_map
        .set(MapRequest::Named { recipe, seed })
        .map_err(BrpError::internal)?;
    Ok(Value::Null)
}

/// `game/player_input {move?, aim?, mainhand?, offhand?}`: holds this input on the player until
/// the next call. `aim` is relative to the player. `{release: true}` returns control to the
/// keyboard and mouse.
fn set_player_input(In(params): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let input: ScriptedPlayerInput = parse_params(params)?;
    // Applied once more with nothing held, so a released offhand is lowered.
    commands.insert_resource(ScriptedPlayerInput {
        movement: input.movement.filter(|_| !input.release),
        aim: input.aim.filter(|_| !input.release),
        mainhand: input.mainhand && !input.release,
        offhand: input.offhand && !input.release,
        release: input.release,
    });
    Ok(Value::Null)
}

fn apply_scripted_player_input(
    mut commands: Commands,
    input: Res<ScriptedPlayerInput>,
    player: Single<(Entity, &mut Kinematics, &mut Vision), With<Player>>,
    mut player_aim: Single<&mut Transform, With<PlayerAim>>,
    mut equipment: EquipmentUsage,
) {
    let (player, mut kinematics, mut vision) = player.into_inner();
    match input.movement.and_then(Vec2::try_normalize) {
        Some(direction) => kinematics.move_in_direction(direction, 1.0),
        None => kinematics.stop_moving(),
    }
    if let Some(aim) = input.aim
        && let Some(direction) = aim.try_normalize()
    {
        vision.aim_direction = direction;
        vision.aim_distance = aim.length();
        player_aim.translation = aim.extend(0.0);
    }
    if input.mainhand && !equipment.is_using(player, EquipmentSlot::Mainhand) {
        // Discarded: a refused use is retried next frame.
        let _ = equipment.try_use(player, EquipmentSlot::Mainhand);
    }
    let blocking = equipment.is_using(player, EquipmentSlot::Offhand);
    if input.offhand && !blocking {
        let _ = equipment.try_use(player, EquipmentSlot::Offhand);
    } else if !input.offhand && blocking {
        equipment.stop_using(player, EquipmentSlot::Offhand);
    }
    if input.release {
        commands.remove_resource::<ScriptedPlayerInput>();
    }
}

/// What `game/key` does with a key.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum KeyAction {
    Press,
    Release,
    Tap,
}

#[derive(Deserialize)]
struct KeyParams {
    key: KeyCode,
    action: KeyAction,
}

/// `game/key {key, action}`: presses, releases or taps a key as if typed into the window, so
/// every binding works, such as `"Space"` to interact, `"KeyI"` for the inventory, `"Escape"` to
/// pause and the dev keys. `key` is a `KeyCode` name and `action` is `press`, `release` or `tap`.
fn press_key(
    In(params): In<Option<Value>>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut keyboard_input: MessageWriter<KeyboardInput>,
    mut pending_key_releases: ResMut<PendingKeyReleases>,
) -> BrpResult {
    let KeyParams { key, action } = parse_params(params)?;
    let state = match action {
        KeyAction::Press | KeyAction::Tap => ButtonState::Pressed,
        KeyAction::Release => ButtonState::Released,
    };
    keyboard_input.write(keyboard_message(*window, key, state));
    if let KeyAction::Tap = action {
        pending_key_releases.push(PendingKeyRelease {
            key,
            frames_left: TAP_FRAMES,
        });
    }
    Ok(Value::Null)
}

fn keyboard_message(window: Entity, key: KeyCode, state: ButtonState) -> KeyboardInput {
    KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window,
    }
}

fn release_tapped_keys(
    window: Option<Single<Entity, With<PrimaryWindow>>>,
    mut keyboard_input: MessageWriter<KeyboardInput>,
    mut pending_key_releases: ResMut<PendingKeyReleases>,
) {
    let Some(window) = window else {
        return;
    };
    for pending in pending_key_releases.iter_mut() {
        pending.frames_left = pending.frames_left.saturating_sub(1);
        if pending.frames_left == 0 {
            keyboard_input.write(keyboard_message(
                *window,
                pending.key,
                ButtonState::Released,
            ));
        }
    }
    pending_key_releases.retain(|pending| pending.frames_left > 0);
}

/// Clicks whose pointer moved last frame, pressed once picking has seen the pointer there.
#[derive(Resource, Debug, Default, Deref, DerefMut)]
struct PendingClicks(Vec<PendingClick>);

/// Where `game/click` presses, and with which button.
#[derive(Clone, Debug)]
struct PendingClick {
    location: Location,
    button: PointerButton,
}

/// Which mouse button `game/click` presses.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ClickButton {
    #[default]
    Primary,
    Secondary,
}

#[derive(Deserialize)]
struct ClickParams {
    position: Vec2,
    #[serde(default)]
    button: ClickButton,
}

/// `game/click {position, button?}`: moves the mouse pointer to `position` in logical window
/// pixels and clicks there, which activates buttons and inventory slots. `button` is `primary`
/// (the default) or `secondary`. `game/ui` lists where text is drawn.
fn click(
    In(params): In<Option<Value>>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut pointer_input: MessageWriter<PointerInput>,
    mut pending_clicks: ResMut<PendingClicks>,
) -> BrpResult {
    let ClickParams { position, button } = parse_params(params)?;
    let button = match button {
        ClickButton::Primary => PointerButton::Primary,
        ClickButton::Secondary => PointerButton::Secondary,
    };
    let window = WindowRef::Primary
        .normalize(Some(*window))
        .ok_or_else(|| BrpError::internal("no primary window"))?;
    let location = Location {
        target: NormalizedRenderTarget::Window(window),
        position,
    };
    pointer_input.write(PointerInput::new(
        PointerId::Mouse,
        location.clone(),
        PointerAction::Move { delta: Vec2::ZERO },
    ));
    pending_clicks.push(PendingClick { location, button });
    Ok(Value::Null)
}

fn press_pending_clicks(
    mut pointer_input: MessageWriter<PointerInput>,
    mut pending_clicks: ResMut<PendingClicks>,
) {
    for PendingClick { location, button } in pending_clicks.drain(..) {
        for action in [PointerAction::Press(button), PointerAction::Release(button)] {
            pointer_input.write(PointerInput::new(
                PointerId::Mouse,
                location.clone(),
                action,
            ));
        }
    }
}

/// `game/ui`: every visible UI text with its center and size in logical window pixels, for
/// `game/click`.
#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
fn list_ui_text(
    In(_): In<Option<Value>>,
    texts: Query<(
        Entity,
        &Text,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
) -> BrpResult {
    let texts: Vec<Value> = texts
        .iter()
        .filter(|(.., visibility)| visibility.get())
        .map(|(entity, text, node, transform, _)| {
            let scale = node.inverse_scale_factor();
            json!({
                "entity": entity.to_bits(),
                "text": text.0,
                "center": (transform.translation * scale).to_array(),
                "size": (node.size() * scale).to_array(),
            })
        })
        .collect();
    Ok(json!(texts))
}

#[derive(Deserialize)]
struct TeleportParams {
    position: Vec2,
}

/// `game/teleport_player {position}`: moves the player to a world position.
fn teleport_player(
    In(params): In<Option<Value>>,
    mut player: Single<&mut Transform, With<Player>>,
) -> BrpResult {
    let TeleportParams { position } = parse_params(params)?;
    player.translation = position.extend(player.translation.z);
    Ok(Value::Null)
}

#[derive(Deserialize)]
struct SpawnEnemiesParams {
    kind: EnemyKind,
    positions: Vec<Vec2>,
}

/// `game/spawn_enemies {kind, positions}`: spawns enemies of one `EnemyKind` at world positions.
fn spawn_enemies(In(params): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let SpawnEnemiesParams { kind, positions } = parse_params(params)?;
    let spawns = positions
        .into_iter()
        .map(|position| EnemySpawn { position, kind })
        .collect();
    commands.trigger(SpawnEnemies(spawns));
    Ok(Value::Null)
}

#[derive(Deserialize)]
struct PlayerHealthParams {
    hp: f32,
    max_hp: f32,
}

/// `game/set_player_health {hp, max_hp}`.
fn set_player_health(
    In(params): In<Option<Value>>,
    mut player_health: Single<&mut Health, With<Player>>,
) -> BrpResult {
    let PlayerHealthParams { hp, max_hp } = parse_params(params)?;
    player_health.max_hp = max_hp;
    player_health.hp = hp.min(max_hp);
    Ok(Value::Null)
}

#[derive(Deserialize)]
struct InvincibleParams {
    invincible: bool,
}

/// `game/set_invincible {invincible}`.
fn set_invincible(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) -> BrpResult {
    let InvincibleParams { invincible } = parse_params(params)?;
    if invincible {
        commands.entity(*player).insert(Invincible);
    } else {
        commands.entity(*player).remove::<Invincible>();
    }
    Ok(Value::Null)
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct TimeParams {
    paused: Option<bool>,
    speed: Option<f32>,
}

/// `game/time {paused?, speed?}`: pauses, resumes or rescales game time, then returns it. Physics
/// and fixed steps follow; menus and real-time tools don't.
fn control_time(
    In(params): In<Option<Value>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) -> BrpResult {
    let TimeParams { paused, speed } = parse_params(params)?;
    match paused {
        Some(true) => virtual_time.pause(),
        Some(false) => virtual_time.unpause(),
        None => {}
    }
    if let Some(speed) = speed {
        virtual_time.set_relative_speed(speed);
    }
    Ok(json!({
        "time": virtual_time.elapsed_secs(),
        "paused": virtual_time.is_paused(),
        "speed": virtual_time.relative_speed(),
    }))
}

#[derive(Deserialize)]
struct ScreenshotParams {
    path: String,
}

/// `game/screenshot {path}`: saves the primary window to an image file a frame or two later.
fn take_screenshot(In(params): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let ScreenshotParams { path } = parse_params(params)?;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
    Ok(Value::Null)
}

/// `game/exit`: closes the game after this frame.
#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
fn exit_game(In(_): In<Option<Value>>, mut app_exit: MessageWriter<AppExit>) -> BrpResult {
    app_exit.write(AppExit::Success);
    Ok(Value::Null)
}

/// `game/snapshot`: the app and menu states, game time, the player, every enemy's brain and
/// movement, projectiles, interactables, portals, and the hits landed since the previous snapshot.
#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
fn take_snapshot(
    In(_): In<Option<Value>>,
    time: Res<Time<Virtual>>,
    app_state: Res<State<AppState>>,
    playing_state: Option<Res<State<PlayingState>>>,
    menu: Res<State<Menu>>,
    mut recorded_hits: ResMut<RecordedHits>,
    player: Option<
        Single<
            (
                Entity,
                &Transform,
                &LinearVelocity,
                Option<&Health>,
                Option<&Mana>,
                &Purse,
                &Player,
                Option<&Mainhand>,
                Option<&Offhand>,
            ),
            With<Player>,
        >,
    >,
    enemies: Query<
        (
            Entity,
            &Transform,
            &Kinematics,
            &StatMultipliers,
            &LinearVelocity,
            &FacingDirection,
            &Health,
            Option<&Mana>,
            Option<&Brain>,
            Option<&Dash>,
            (&TargetInfo, Has<Targeting>, Has<TargetLock>, &Attacking),
        ),
        With<Enemy>,
    >,
    projectiles: Query<&Transform, With<Projectile>>,
    interaction_zones: Query<(&ChildOf, &GlobalTransform), With<InteractionZone>>,
    names: Query<&Name>,
    portals: Query<&Transform, With<Portal>>,
) -> BrpResult {
    let player = player.map(|player| {
        let (entity, transform, velocity, health, mana, purse, player, mainhand, offhand) =
            player.into_inner();
        let item_name = |item: Entity| names.get(item).ok().map(Name::as_str);
        json!({
            "entity": entity.to_bits(),
            "position": transform.translation.truncate().to_array(),
            "velocity": velocity.0.to_array(),
            "hp": health.map(|health| health.hp),
            "max_hp": health.map(|health| health.max_hp),
            "mana": mana.map(|mana| mana.current),
            "gold": purse.amount,
            "level": player.level(),
            "mainhand": mainhand.and_then(|mainhand| item_name(mainhand.get())),
            "offhand": offhand.and_then(|offhand| item_name(offhand.get())),
        })
    });
    let enemies: Vec<Value> = enemies
        .iter()
        .map(
            |(
                entity,
                transform,
                kinematics,
                stat_multipliers,
                velocity,
                facing_direction,
                health,
                mana,
                brain,
                dash,
                (target_info, has_target, has_target_lock, attacking),
            )| {
                let kind = brain.and_then(|brain| {
                    EnemyKind::ALL
                        .iter()
                        .find(|kind| std::ptr::eq(kind.brain(), brain.definition()))
                });
                let motion = brain.and_then(Brain::motion).map(|motion| match motion {
                    Motion::Dash => dash.map_or(motion.name(), Dash::name),
                    other => other.name(),
                });
                json!({
                    "entity": entity.to_bits(),
                    "kind": kind.map(|kind| format!("{kind:?}")),
                    "position": transform.translation.truncate().to_array(),
                    "intended_velocity": kinematics.velocity(stat_multipliers.speed).to_array(),
                    "velocity": velocity.0.to_array(),
                    "facing": format!("{facing_direction:?}"),
                    "stunned": stat_multipliers.is_stunned(),
                    "hp": health.hp,
                    "max_hp": health.max_hp,
                    "mana": mana.map(|mana| mana.current),
                    "motion": motion,
                    "action": brain.and_then(Brain::action).map(|action| format!("{action:?}")),
                    "has_target": has_target,
                    "target_lock": has_target_lock,
                    "line_of_sight": target_info.line_of_sight,
                    "target_distance": target_info.distance,
                    "attacking": **attacking,
                })
            },
        )
        .collect();
    let projectiles: Vec<Value> = projectiles
        .iter()
        .map(|transform| json!(transform.translation.truncate().to_array()))
        .collect();
    let interactables: Vec<Value> = interaction_zones
        .iter()
        .map(|(child_of, transform)| {
            json!({
                "entity": child_of.parent().to_bits(),
                "name": names.get(child_of.parent()).ok().map(Name::as_str),
                "position": transform.translation().truncate().to_array(),
            })
        })
        .collect();
    let portals: Vec<Value> = portals
        .iter()
        .map(|transform| json!(transform.translation.truncate().to_array()))
        .collect();
    Ok(json!({
        "time": time.elapsed_secs(),
        "paused": time.is_paused(),
        "state": format!("{:?}", app_state.get()),
        "playing_state": playing_state.map(|state| format!("{:?}", state.get())),
        "menu": format!("{:?}", menu.get()),
        "player": player,
        "enemies": enemies,
        "projectiles": projectiles,
        "interactables": interactables,
        "portals": portals,
        "hits": std::mem::take(&mut **recorded_hits),
    }))
}
