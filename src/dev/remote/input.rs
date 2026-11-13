use bevy::camera::NormalizedRenderTarget;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_enhanced_input::prelude::{
    ActionValue, InputAction, MockEntityCommandsExt, MockSpan, TriggerState,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::json::{entity_id, parse_params};
use crate::prelude::*;

/// Frames a tapped key stays down, so actions that read held keys see it.
const TAP_FRAMES: u32 = 2;
/// Frames a key stays up between two scripted presses, so the second reads as a new press.
const REPRESS_FRAMES: u32 = 1;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PendingKeyEvents>()
        .init_resource::<PendingClicks>()
        // In `Last`, so they run in menus too. Remote methods run after `Last`, so these see a
        // request on the next frame.
        .add_systems(Last, (send_pending_key_events, press_pending_clicks));
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct PlayerInputParams {
    #[serde(rename = "move")]
    movement: Option<Vec2>,
    aim: Option<Vec2>,
    mainhand: bool,
    offhand: bool,
}

/// `game/player_input {move?, aim?, mainhand?, offhand?}`: drives the player's own input actions,
/// so the game reacts as it does to the keyboard and mouse. `move` and `offhand` stay held until
/// the next call, and a call without them lets go. `mainhand` is one click per call, as long as a
/// frame passes between calls. `aim` moves the reticle to an offset from the player.
pub(super) fn set_player_input(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    player: Single<(Entity, &mut Vision), With<Player>>,
    mut player_aim: Single<&mut Transform, With<PlayerAim>>,
) -> BrpResult {
    let PlayerInputParams {
        movement,
        aim,
        mainhand,
        offhand,
    } = parse_params(params)?;
    let (player, mut vision) = player.into_inner();
    let mut player = commands.entity(player);

    match movement.and_then(Vec2::try_normalize) {
        Some(direction) => hold_action::<PlayerMovement>(&mut player, direction),
        None => release_action::<PlayerMovement>(&mut player, Vec2::ZERO),
    }

    if offhand {
        hold_action::<UseOffhand>(&mut player, true);
    } else {
        release_action::<UseOffhand>(&mut player, false);
    }

    if mainhand {
        player.mock_once::<Player, UseMainhand>(TriggerState::Fired, true);
    }

    if let Some(aim) = aim
        && let Some(direction) = aim.try_normalize()
    {
        vision.aim_direction = direction;
        vision.aim_distance = aim.length();
        player_aim.translation = aim.extend(0.0);
    }

    Ok(Value::Null)
}

/// Keeps a player action firing with `value` until [`release_action`] ends it.
fn hold_action<A: InputAction>(player: &mut EntityCommands, value: impl Into<ActionValue>) {
    player.mock::<Player, A>(TriggerState::Fired, value, MockSpan::Manual);
}

/// Ends a held player action, then hands the action back to the keyboard and mouse.
/// `idle_value` is the action's value at rest.
fn release_action<A: InputAction>(player: &mut EntityCommands, idle_value: impl Into<ActionValue>) {
    player.mock_once::<Player, A>(TriggerState::None, idle_value);
}

/// Key presses and releases from `game/key` waiting for their frame.
#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub(super) struct PendingKeyEvents(Vec<PendingKeyEvent>);

/// A key press or release sent `frames_left` frames from now.
#[derive(Clone, Copy, Debug)]
pub(super) struct PendingKeyEvent {
    key: KeyCode,
    state: ButtonState,
    frames_left: u32,
}

impl PendingKeyEvents {
    /// Frames until `key` can be pressed again: right away, or [`REPRESS_FRAMES`] after its
    /// last queued event.
    fn frames_until_key_is_free(&self, key: KeyCode) -> u32 {
        self.iter()
            .filter(|pending| pending.key == key)
            .map(|pending| pending.frames_left + REPRESS_FRAMES)
            .max()
            .unwrap_or(0)
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
/// A key still held by an earlier tap is released first, so every tap counts as a new press.
pub(super) fn press_key(
    In(params): In<Option<Value>>,
    mut pending_key_events: ResMut<PendingKeyEvents>,
) -> BrpResult {
    let KeyParams { key, action } = parse_params(params)?;

    // The soonest an event can be sent is the next frame.
    let first_frame = 1 + pending_key_events.frames_until_key_is_free(key);
    let mut send_in = |frames_left, state| {
        pending_key_events.push(PendingKeyEvent {
            key,
            state,
            frames_left,
        });
    };

    match action {
        KeyAction::Press => send_in(first_frame, ButtonState::Pressed),
        KeyAction::Release => send_in(first_frame, ButtonState::Released),
        KeyAction::Tap => {
            send_in(first_frame, ButtonState::Pressed);
            send_in(first_frame + TAP_FRAMES, ButtonState::Released);
        }
    }

    Ok(Value::Null)
}

fn send_pending_key_events(
    window: Single<Entity, With<PrimaryWindow>>,
    mut keyboard_input: MessageWriter<KeyboardInput>,
    mut pending_key_events: ResMut<PendingKeyEvents>,
) {
    for pending in pending_key_events.iter_mut() {
        pending.frames_left -= 1;

        if pending.frames_left == 0 {
            keyboard_input.write(KeyboardInput {
                key_code: pending.key,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state: pending.state,
                text: None,
                repeat: false,
                window: *window,
            });
        }
    }

    pending_key_events.retain(|pending| pending.frames_left > 0);
}

/// Clicks whose pointer moved last frame, pressed once picking has seen the pointer there.
#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub(super) struct PendingClicks(Vec<PendingClick>);

/// Where `game/click` presses, and with which button.
#[derive(Clone, Debug)]
pub(super) struct PendingClick {
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
pub(super) fn click(
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

    let move_pointer = PointerAction::Move { delta: Vec2::ZERO };
    pointer_input.write(PointerInput::new(
        PointerId::Mouse,
        location.clone(),
        move_pointer,
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
pub(super) fn list_ui_text(
    In(_): In<Option<Value>>,
    texts: Query<(
        Entity,
        &Text,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
) -> BrpResult {
    let mut visible_texts = Vec::new();

    for (entity, text, node, transform, visibility) in &texts {
        if !visibility.get() {
            continue;
        }
        let logical_pixels = node.inverse_scale_factor();

        visible_texts.push(json!({
            "entity": entity_id(entity),
            "text": text.0,
            "center": (transform.translation * logical_pixels).to_array(),
            "size": (node.size() * logical_pixels).to_array(),
        }));
    }

    Ok(json!(visible_texts))
}
