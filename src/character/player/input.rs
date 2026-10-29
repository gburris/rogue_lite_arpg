use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::aim::AimInput;
use super::interact::PlayerInteractionInput;
use super::movement::PlayerMovement;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();

    // Use Equipment Logic
    app.add_observer(on_use_mainhand_input)
        .add_observer(on_use_offhand_input)
        .add_observer(on_use_offhand_complete);

    // Pause Logic
    app.add_systems(OnEnter(Pause(false)), set_controls(true))
        .add_systems(OnEnter(Pause(true)), set_controls(false))
        .add_systems(OnEnter(PlayingState::Death), set_controls(false))
        .add_observer(on_inventory_opened);
}

const MOUSE_SENSITIVITY: f32 = 0.5;
const CONTROLLER_AIM_SENSITIVITY: f32 = 8.0;

pub(super) fn player_actions() -> impl Bundle {
    (actions!(Player[
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
            bindings![MouseButton::Left, GamepadButton::RightTrigger, GamepadButton::RightTrigger2],
        ),
        (
            Action::<UseOffhand>::new(),
            bindings![MouseButton::Right, GamepadButton::LeftTrigger, GamepadButton::LeftTrigger2],
        )
    ]),)
}

#[derive(InputAction)]
#[action_output(bool)]
struct OpenInventory;

fn on_inventory_opened(_: On<Start<OpenInventory>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::Inventory);
}

fn set_controls(active: bool) -> impl Fn(Commands, Single<Entity, With<Player>>) {
    move |mut commands, player| {
        let activity = if active {
            ContextActivity::<Player>::ACTIVE
        } else {
            ContextActivity::<Player>::INACTIVE
        };

        commands.entity(*player).insert(activity);
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct UseMainhand;

#[derive(InputAction)]
#[action_output(bool)]
struct UseOffhand;

fn on_use_mainhand_input(use_mainhand: On<Start<UseMainhand>>, mut equipment: EquipmentUsage) {
    // Discarded: failures are already queued as `EquipmentUseFailed` for the HUD.
    let _ = equipment.try_use(use_mainhand.context, EquipmentSlot::Mainhand);
}

fn on_use_offhand_input(use_offhand: On<Start<UseOffhand>>, mut equipment: EquipmentUsage) {
    // Discarded: failures are already queued as `EquipmentUseFailed` for the HUD.
    let _ = equipment.try_use(use_offhand.context, EquipmentSlot::Offhand);
}

fn on_use_offhand_complete(use_offhand: On<Complete<UseOffhand>>, mut equipment: EquipmentUsage) {
    equipment.stop_using(use_offhand.context, EquipmentSlot::Offhand);
}
