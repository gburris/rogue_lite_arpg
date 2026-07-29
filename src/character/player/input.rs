use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    character::player::{AimInput, PlayerMovement, overlay::PlayerEquipmentUsed},
    prelude::*,
};

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

fn on_use_mainhand_input(
    use_mainhand: On<Start<UseMainhand>>,
    mut commands: Commands,
    player: Option<Single<(&Mainhand, Option<&mut Mana>), With<Player>>>,
    mut mainhand_query: Query<EquipmentUsed>,
) {
    let Some(player) = player else {
        commands.trigger(EquipmentUseFailed {
            holder: use_mainhand.context,
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
            holder: use_mainhand.context,
            slot: EquipmentSlot::Mainhand,
            reason: failure_reason,
        });
    } else {
        commands.trigger(UseEquipment {
            entity: mainhand.get(),
        });
        commands.trigger(PlayerEquipmentUsed {
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
            holder: use_offhand.context,
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
            holder: use_offhand.context,
            slot: EquipmentSlot::Offhand,
            reason: failure_reason,
        });
    } else {
        commands.trigger(UseEquipment {
            entity: offhand.get(),
        });
        commands.trigger(PlayerEquipmentUsed {
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
        });
    }
}
