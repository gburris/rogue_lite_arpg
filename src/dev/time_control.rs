//! Dev keys for game time: F5 pauses or resumes, F7 steps while paused, and F8 and F9 halve and
//! double the speed. They change [`Time<Virtual>`], so fixed steps and physics follow, while the
//! pause menu and real-time tools like hitch detection are unaffected.

use std::time::Duration;

use bevy::prelude::*;

const SLOWEST_SPEED: f32 = 1.0 / 8.0;
const FASTEST_SPEED: f32 = 8.0;

pub(super) fn plugin(app: &mut App) {
    // In PostUpdate, so writing `Time<Virtual>` doesn't race the Update systems that read it.
    app.add_systems(
        PostUpdate,
        (toggle_pause, run_one_fixed_step, change_speed).chain(),
    );
}

fn toggle_pause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F5) {
        return;
    }
    if virtual_time.is_paused() {
        virtual_time.unpause();
        info!("time resumed");
    } else {
        virtual_time.pause();
        info!("time paused");
    }
}

/// Unpauses until at least one fixed step has run, then pauses again. Pausing with F5 meanwhile
/// ends the step.
fn run_one_fixed_step(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    fixed_time: Res<Time<Fixed>>,
    mut step_started_at: Local<Option<Duration>>,
) {
    if let Some(started_at) = *step_started_at
        && (virtual_time.is_paused() || fixed_time.elapsed() > started_at)
    {
        virtual_time.pause();
        *step_started_at = None;
    }
    if keyboard_input.just_pressed(KeyCode::F7) && virtual_time.is_paused() {
        virtual_time.unpause();
        *step_started_at = Some(fixed_time.elapsed());
    }
}

fn change_speed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    let factor = if keyboard_input.just_pressed(KeyCode::F8) {
        0.5
    } else if keyboard_input.just_pressed(KeyCode::F9) {
        2.0
    } else {
        return;
    };
    let speed = (virtual_time.relative_speed() * factor).clamp(SLOWEST_SPEED, FASTEST_SPEED);
    virtual_time.set_relative_speed(speed);
    info!("time speed {speed}x");
}
