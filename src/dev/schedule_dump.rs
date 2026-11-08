//! Writes the `Update` and `FixedUpdate` schedule graphs as Graphviz DOT files.

use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy_mod_debugdump::schedule_graph::Settings;

/// Directory the DOT files are written to.
const OUTPUT_DIR: &str = "runs/schedules";

/// Writes the schedule graphs for `--dump-schedules`. Render one with
/// `dot -Tsvg runs/schedules/update.dot -o update.svg`.
pub(crate) fn dump_schedules(app: &mut App) {
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
}

fn write_graph(file_name: &str, dot: &str) {
    let path = Path::new(OUTPUT_DIR).join(file_name);
    fs::write(&path, dot).expect("schedule graph file should be writable");
}
