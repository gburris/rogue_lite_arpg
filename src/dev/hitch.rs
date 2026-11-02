//! F4 marks a hitch: it sends a Tracy message under `profile` and writes the last 10 seconds of
//! every diagnostic to `runs/diagnostics_<milliseconds since the Unix epoch>.csv`.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::prelude::*;

/// How far back a dump reaches.
const HISTORY_SECONDS: f32 = 10.0;

const OUTPUT_DIRECTORY: &str = "runs";

/// Every diagnostic's value in one frame, indexed like [`DiagnosticHistory::paths`]. A diagnostic
/// that had no value, or did not exist yet, is NaN or past the end.
struct FrameValues {
    elapsed_seconds: f32,
    values: Vec<f64>,
}

/// The last [`HISTORY_SECONDS`] of frames. Paths are stored once, so recording a frame costs one
/// allocation.
#[derive(Resource, Default)]
struct DiagnosticHistory {
    paths: Vec<DiagnosticPath>,
    index_by_path: HashMap<DiagnosticPath, usize>,
    frames: VecDeque<FrameValues>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DiagnosticHistory>();
    app.add_systems(
        Update,
        (
            record_diagnostics.after(InGameSystems::HudOverlay),
            mark_hitch,
        )
            .chain(),
    );
}

fn record_diagnostics(
    real_time: Res<Time<Real>>,
    diagnostics_store: Res<DiagnosticsStore>,
    mut history: ResMut<DiagnosticHistory>,
) {
    let elapsed_seconds = real_time.elapsed_secs();
    let mut values = vec![f64::NAN; history.paths.len()];
    for diagnostic in diagnostics_store.iter() {
        let path = diagnostic.path();
        let index = match history.index_by_path.get(path) {
            Some(&index) => index,
            None => {
                let index = history.paths.len();
                history.paths.push(path.clone());
                history.index_by_path.insert(path.clone(), index);
                values.push(f64::NAN);
                index
            }
        };
        if let Some(value) = diagnostic.value() {
            values[index] = value;
        }
    }
    history.frames.push_back(FrameValues {
        elapsed_seconds,
        values,
    });
    while history
        .frames
        .front()
        .is_some_and(|frame| frame.elapsed_seconds < elapsed_seconds - HISTORY_SECONDS)
    {
        history.frames.pop_front();
    }
}

fn mark_hitch(keyboard_input: Res<ButtonInput<KeyCode>>, history: Res<DiagnosticHistory>) {
    if !keyboard_input.just_pressed(KeyCode::F4) {
        return;
    }
    #[cfg(feature = "profile")]
    if let Some(client) = tracy_client::Client::running() {
        client.message("hitch here", 0);
    }
    match write_csv(&history) {
        Ok(path) => info!("Wrote the last {HISTORY_SECONDS} s of diagnostics to {path}"),
        Err(error) => error!("Could not write the diagnostics dump: {error}"),
    }
}

/// Writes one row per frame and one column per diagnostic, returning the file's path.
fn write_csv(history: &DiagnosticHistory) -> std::io::Result<String> {
    let mut csv = String::from("elapsed_seconds");
    for path in &history.paths {
        write!(csv, ",{}", path.as_str()).expect("writing to a String cannot fail");
    }
    csv.push('\n');
    for frame in &history.frames {
        write!(csv, "{:.4}", frame.elapsed_seconds).expect("writing to a String cannot fail");
        for column in 0..history.paths.len() {
            csv.push(',');
            if let Some(value) = frame.values.get(column).filter(|value| !value.is_nan()) {
                write!(csv, "{value}").expect("writing to a String cannot fail");
            }
        }
        csv.push('\n');
    }

    let milliseconds_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_millis();
    let path = format!("{OUTPUT_DIRECTORY}/diagnostics_{milliseconds_since_epoch}.csv");
    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    fs::write(&path, csv)?;
    Ok(path)
}
