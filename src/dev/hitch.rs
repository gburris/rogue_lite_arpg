//! F4 logs a hitch marker, and any frame at least [`HITCH_FACTOR`] times the recent average and
//! slower than [`MINIMUM_HITCH_MS`] is logged as a hitch. Under `tracy` each log line is also a
//! Tracy message. A detected hitch lands two frames after the slow frame, since with pipelined
//! rendering each frame's delta measures the main frame two before it.

use bevy::prelude::*;

/// How many times the recent average frame time a frame must take to count as a hitch.
const HITCH_FACTOR: f32 = 3.0;

/// Frames faster than 60 fps are never hitches, so jitter at high frame rates isn't logged.
const MINIMUM_HITCH_MS: f32 = 1000.0 / 60.0;

/// Weight of the newest frame in [`RecentFrameTime`].
const AVERAGE_SMOOTHING: f32 = 0.05;

/// Exponential moving average of real frame time, which [`detect_hitches`] compares against.
#[derive(Resource, Clone, Copy, Debug, Default)]
struct RecentFrameTime {
    average_ms: Option<f32>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RecentFrameTime>();
    app.add_systems(Update, mark_hitch);
    app.add_systems(Last, detect_hitches);
}

/// Reports a frame that took far longer than the recent average, then updates the average.
fn detect_hitches(real_time: Res<Time<Real>>, mut recent_frame_time: ResMut<RecentFrameTime>) {
    let frame_ms = real_time.delta_secs() * 1000.0;
    if frame_ms <= 0.0 {
        return;
    }
    let Some(average_ms) = recent_frame_time.average_ms else {
        recent_frame_time.average_ms = Some(frame_ms);
        return;
    };

    if frame_ms >= MINIMUM_HITCH_MS && frame_ms >= average_ms * HITCH_FACTOR {
        info!(
            "hitch: {frame_ms:.1} ms frame ending at {:.2} s, average {average_ms:.1} ms",
            real_time.elapsed_secs()
        );
    }

    // Cap the frame at the threshold, so one long frame can't hide the next, but a lasting
    // slowdown still raises the average.
    let capped_ms = frame_ms.min(average_ms * HITCH_FACTOR);
    recent_frame_time.average_ms = Some(average_ms + (capped_ms - average_ms) * AVERAGE_SMOOTHING);
}

fn mark_hitch(keyboard_input: Res<ButtonInput<KeyCode>>, real_time: Res<Time<Real>>) {
    if keyboard_input.just_pressed(KeyCode::F4) {
        info!("hitch marked at {:.2} s", real_time.elapsed_secs());
    }
}
