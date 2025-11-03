use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    character::player::{AimInput, PlayerMovement},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();

    // Use Equipment Logic
    app.add_observer(on_use_mainhand_input)
        .add_observer(on_use_offhand_input)
        .add_observer(on_use_offhand_complete);

    // Pause Logic
    app.add_observer(on_pause)
        .add_systems(OnEnter(Pause(true)), deactivate_controls)
        .add_observer(on_inventory_opened);

    // Unpause Logic
    app.add_systems(OnEnter(Menu::None), unpause)
        .add_observer(on_controls_activated);
}

const MOUSE_SENSITIVITY: f32 = 0.5;
const CONTROLLER_AIM_SENSITIVITY: f32 = 8.0;

pub(super) fn player_actions() -> impl Bundle {
    (actions!(Player[
        (
            Action::<PauseGame>::new(),
            // We set `require_reset` to `true` because `ResumeGame` action uses the same input,
            // and we want it to be triggerable only after the button is released.
            ActionSettings {
                require_reset: true,
                ..Default::default()
            },
            // Can't allow escape on wasm because that un-grabs cursor
            Bindings::spawn((
                Spawn(Binding::from(KeyCode::KeyP)),
                Spawn(Binding::from(GamepadButton::Start)),
                #[cfg(not(target_family = "wasm"))]
                Spawn(Binding::from(KeyCode::Escape))
            )),
        ),
        (
            Action::<OpenInventory>::new(),
            bindings![KeyCode::KeyI],
        ),
        (
            Action::<PlayerInteractionInput>::new(),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<PlayerMovement>::new(),
            DeadZone::default(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick(),
            )),
        ),
        (
            Action::<AimInput>::new(),
            Bindings::spawn((
                Spawn((Binding::mouse_motion(), Negate::y(), Scale::splat(MOUSE_SENSITIVITY))),
                Axial::right_stick().with((
                    DeadZone { upper_threshold: 0.8, ..default() },
                    Scale::splat(CONTROLLER_AIM_SENSITIVITY),
                ))
            )),
        ),
        (
            Action::<UseMainhand>::new(),
            bindings![MouseButton::Left, GamepadButton::RightTrigger],
        ),
        (
            Action::<UseOffhand>::new(),
            bindings![MouseButton::Right, GamepadButton::LeftTrigger],
        )
    ]),)
}

#[derive(InputAction)]
#[action_output(bool)]

struct OpenInventory;

fn on_inventory_opened(_: On<Start<OpenInventory>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::Inventory);
}

#[derive(InputAction)]
#[action_output(bool)]

struct PauseGame;

fn on_pause(_: On<Start<PauseGame>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::Pause);
}

fn deactivate_controls(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands
        .entity(*player)
        .insert(ContextActivity::<Player>::INACTIVE);
}

#[derive(Event)]
struct ActivatePlayerControls;

fn unpause(mut commands: Commands, mut next_pause_state: ResMut<NextState<Pause>>) {
    next_pause_state.set(Pause(false));
    commands.trigger(ActivatePlayerControls);
}

fn on_controls_activated(
    _: On<ActivatePlayerControls>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    commands
        .entity(*player)
        .insert(ContextActivity::<Player>::ACTIVE);
}

#[derive(InputAction)]
#[action_output(bool)]
struct UseMainhand;

#[derive(InputAction)]
#[action_output(bool)]
struct UseOffhand;

fn on_use_mainhand_input(
    use_mainhand: On<Start<UseMainhand>>,
    mut commands: Commands,
    player: Option<Single<(&Mainhand, Option<&mut Mana>), With<Player>>>,
    mut mainhand_query: Query<EquipmentUsed>,
) {
    let Some(player) = player else {
        commands.trigger(EquipmentUseFailed {
            entity: use_mainhand.context,
            slot: EquipmentSlot::Mainhand,
            reason: EquipmentUseFailure::NoneEquipped,
        });
        return;
    };

    let (mainhand, mut mana) = player.into_inner();

    let failure_reason = mainhand_query
        .get_mut(mainhand.get())
        .expect("Player should have mainhand")
        .attempt_use(mana.as_deref_mut());

    if let Err(failure_reason) = failure_reason {
        commands.trigger(EquipmentUseFailed {
            entity: use_mainhand.context,
            slot: EquipmentSlot::Mainhand,
            reason: failure_reason,
        });
    } else {
        commands.trigger(UseEquipment {
            entity: mainhand.get(),
        });
    }
}

fn on_use_offhand_input(
    use_offhand: On<Start<UseOffhand>>,
    mut commands: Commands,
    player: Option<Single<(&Offhand, Option<&mut Mana>), With<Player>>>,
    mut offhand_query: Query<EquipmentUsed>,
) {
    let Some(player) = player else {
        commands.trigger(EquipmentUseFailed {
            entity: use_offhand.context,
            slot: EquipmentSlot::Offhand,
            reason: EquipmentUseFailure::NoneEquipped,
        });
        return;
    };

    let (offhand, mut mana) = player.into_inner();

    let failure_reason = offhand_query
        .get_mut(offhand.get())
        .expect("Player should have offhand")
        .attempt_use(mana.as_deref_mut());

    if let Err(failure_reason) = failure_reason {
        commands.trigger(EquipmentUseFailed {
            entity: use_offhand.context,
            slot: EquipmentSlot::Offhand,
            reason: failure_reason,
        });
    } else {
        commands.trigger(UseEquipment {
            entity: offhand.get(),
        });
    }
}

fn on_use_offhand_complete(
    _: On<Complete<UseOffhand>>,
    mut commands: Commands,
    player_offhand: Option<Single<&Offhand, With<Player>>>,
) {
    if let Some(player_offhand) = player_offhand {
        commands.trigger(StopUsingEquipment {
            entity: player_offhand.get(),
        })
    }
}
