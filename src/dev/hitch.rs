//! F4 marks a hitch: it sends a Tracy message under `profile` and writes the last 10 seconds of
//! every diagnostic to `runs/diagnostics_<seconds since the Unix epoch>.csv`.

use std::collections::{BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};
use bevy::prelude::*;

use crate::prelude::*;

/// How far back a dump reaches.
const HISTORY_SECONDS: f32 = 10.0;

const OUTPUT_DIRECTORY: &str = "runs";

/// Every diagnostic's value in one frame.
struct FrameValues {
    elapsed_seconds: f32,
    values: Vec<(DiagnosticPath, f64)>,
}

/// The last [`HISTORY_SECONDS`] of frames.
#[derive(Resource, Default)]
struct DiagnosticHistory {
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
    let values = diagnostics_store
        .iter()
        .filter_map(|diagnostic| Some((diagnostic.path().clone(), diagnostic.value()?)))
        .collect();
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
    let paths: BTreeSet<&str> = history
        .frames
        .iter()
        .flat_map(|frame| frame.values.iter().map(|(path, _)| path.as_str()))
        .collect();
    let mut csv = String::from("elapsed_seconds");
    for path in &paths {
        write!(csv, ",{path}").expect("writing to a String cannot fail");
    }
    csv.push('\n');
    for frame in &history.frames {
        write!(csv, "{:.4}", frame.elapsed_seconds).expect("writing to a String cannot fail");
        for path in &paths {
            csv.push(',');
            if let Some((_, value)) = frame
                .values
                .iter()
                .find(|(frame_path, _)| frame_path.as_str() == *path)
            {
                write!(csv, "{value}").expect("writing to a String cannot fail");
            }
        }
        csv.push('\n');
    }

    let seconds_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_secs();
    let path = format!("{OUTPUT_DIRECTORY}/diagnostics_{seconds_since_epoch}.csv");
    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    fs::write(&path, csv)?;
    Ok(path)
}
