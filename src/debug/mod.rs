use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore, EntityCountDiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin, RegisterDiagnostic,
    },
    ecs::entity::Entities,
    input::common_conditions::input_toggle_active,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector::hierarchy::{Hierarchy, SelectedEntities},
    egui::{self, Color32, RichText},
    DefaultInspectorConfigPlugin,
};
use egui_plot::{Line, Plot, PlotBounds, PlotPoint, Text};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultInspectorConfigPlugin,
            (
                // Diagnostics Plugin Group
                FrameTimeDiagnosticsPlugin,
                EntityDiagnosticsPlugin,
            ),
            EguiPlugin,
        ))
        .add_systems(
            Update,
            (
                inspector_ui.run_if(input_toggle_active(true, KeyCode::Backquote)),
                diagnostics_ui,
            )
                .chain(),
        );
    }
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        // NOTE: this panics if query result is not exactly one
        .single(world)
        .clone();
    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Hierarchy");

                let type_registry = world.resource::<AppTypeRegistry>().clone();
                let type_registry = type_registry.read();
                Hierarchy {
                    world,
                    type_registry: &type_registry,
                    selected: &mut selected_entities,
                    context_menu: None,
                    shortcircuit_entity: None,
                    extra_state: &mut (),
                }
                // FIXME: There's a fuck ton of sprites
                // for some reason which lags everything
                .show::<Without<Sprite>>(ui);
                //.show::<()>(ui);

                ui.label("Press `~` to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
}

fn diagnostics_ui(
    egui_context: Query<&mut EguiContext, With<PrimaryWindow>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut egui_context = egui_context.single().clone();
    egui::Window::new("Diagnostics")
        .default_size((256., 128.))
        .show(egui_context.get_mut(), |ui| {
            let plot = Plot::new("fps")
                .width(128.)
                .view_aspect(2.)
                .y_axis_label("fps")
                .show_axes([false, false]);

            // FPS counter + Plot
            plot.show(ui, |plt_ui| {
                let diagnostic = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
                if let Some(fps) = diagnostic {
                    render_fps_graph(plt_ui, fps);
                }
            });

            // Entity Count
            if let Some(entity_count) =
                diagnostics.get_measurement(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
            {
                ui.label(format!("# entities = {}", entity_count.value));
            }
            ui.allocate_space(ui.available_size())
        });
}

fn render_fps_graph(plt_ui: &mut egui_plot::PlotUi, fps: &Diagnostic) {
    let values: Vec<[f64; 2]> = fps
        .values()
        .enumerate()
        .map(|(i, &v)| [i as f64, v])
        .collect();
    plt_ui.set_plot_bounds(PlotBounds::from_min_max(
        // TODO: hardcoded values
        [0., 0.],
        [fps.get_max_history_length() as f64, 120.],
    ));
    plt_ui.line(Line::new(values).fill(0.));

    let avg_value = fps.average().unwrap_or_default();
    plt_ui.text(Text::new(
        // TODO: proper alignment
        PlotPoint::new(24., 24.),
        RichText::new(format!("{avg_value:0.0}"))
            .size(16.)
            .color(Color32::WHITE),
    ));
}

pub struct EntityDiagnosticsPlugin;
impl EntityDiagnosticsPlugin {
    pub const ENTITY_COUNT: DiagnosticPath = DiagnosticPath::const_new("entity_count");

    pub fn diagnostic_system(mut diagnostics: Diagnostics, entities: &Entities) {
        diagnostics.add_measurement(&Self::ENTITY_COUNT, || entities.len() as f64);
    }
}
impl Plugin for EntityDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::ENTITY_COUNT))
            .add_systems(Update, Self::diagnostic_system);
    }
}
