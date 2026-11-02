//! Writes the `Update` and `FixedUpdate` schedule graphs as Graphviz DOT files.

use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy_mod_debugdump::schedule_graph::Settings;

/// Directory the DOT files are written to.
const OUTPUT_DIR: &str = "runs/schedules";

/// Writes the schedule graphs and returns true when `DUMP_SCHEDULES` is set, so `main` can exit
/// instead of running the game. Render with `dot -Tsvg runs/schedules/update.dot -o update.svg`.
pub fn dump_schedules_if_requested(app: &mut App) -> bool {
    if std::env::var_os("DUMP_SCHEDULES").is_none() {
        return false;
    }
    // Plugins add their final systems in `finish`, so the graph is only complete afterwards.
    app.finish();
    app.cleanup();

    fs::create_dir_all(OUTPUT_DIR).expect("schedule dump directory should be creatable");
    let settings = Settings::default();
    write_graph(
        "update.dot",
        &bevy_mod_debugdump::schedule_graph_dot(app, Update, &settings),
    );
    write_graph(
        "fixed_update.dot",
        &bevy_mod_debugdump::schedule_graph_dot(app, FixedUpdate, &settings),
    );
    info!("wrote schedule graphs to {OUTPUT_DIR}");
    true
}

fn write_graph(file_name: &str, dot: &str) {
    let path = Path::new(OUTPUT_DIR).join(file_name);
    fs::write(&path, dot).expect("schedule graph file should be writable");
}
