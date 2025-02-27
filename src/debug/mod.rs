use crate::map::systems::zone::ZoneBackground;
use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore, FrameTimeDiagnosticsPlugin,
        RegisterDiagnostic,
    },
    ecs::entity::Entities,
    input::common_conditions::input_toggle_active,
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
    window::PrimaryWindow,
};
use bevy_ecs_tilemap::map::TilemapId;
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector::hierarchy::{Hierarchy, SelectedEntities},
    egui::{self, Color32, RichText},
    DefaultInspectorConfigPlugin,
};
use egui_extras::Column;
use egui_plot::{Line, Plot, PlotBounds, PlotPoint, Text};
const ENTITY_COUNT: DiagnosticPath = EntityDiagnosticsPlugin::ENTITY_COUNT;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin,
            (
                // Diagnostics Plugin Group
                DefaultInspectorConfigPlugin,
                FrameTimeDiagnosticsPlugin,
                EntityDiagnosticsPlugin,
                RenderDiagnosticsPlugin,
            ),
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
    #[cfg(feature = "trace")]
    let _span = info_span!("debug/ui", name = "inspector_ui").entered();
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                #[cfg(feature = "trace")]
                let _span = info_span!("debug/ui", name = "hierarchy_panel").entered();

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
                .show::<(Without<TilemapId>, Without<ZoneBackground>)>(ui);

                ui.label("Press `~` to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                #[cfg(feature = "trace")]
                let _span = info_span!("debug/ui", name = "inspector_panel").entered();

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
    egui_context: Single<&mut EguiContext, With<PrimaryWindow>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    #[cfg(feature = "trace")]
    let _span = info_span!("debug/ui", name = "diagnostics_ui").entered();

    let mut egui_context = egui_context.clone();

    egui::Window::new("Diagnostics")
        .default_size((256., 128.))
        .show(egui_context.get_mut(), |ui| {
            ui_plot(ui, &diagnostics);
            // Entity Count
            let entity_count = diagnostics
                .get_measurement(&ENTITY_COUNT)
                .map(|d| d.value)
                .unwrap_or_default();
            ui.label(format!("# entities = {:}", entity_count));

            // Diagnostics Table
            ui_diagnostics_table(ui, diagnostics);

            ui.allocate_space(ui.available_size())
        });
}

fn ui_diagnostics_table(ui: &mut egui::Ui, diagnostics: Res<DiagnosticsStore>) {
    #[cfg(feature = "trace")]
    let _span = info_span!("debug/ui", name = "ui_diagnostics_table").entered();
    egui_extras::TableBuilder::new(ui)
        .id_salt("diagnostics_table")
        .resizable(true)
        .striped(true)
        .column(Column::auto())
        .column(Column::remainder())
        .header(12., |mut h| {
            h.col(|ui| {
                ui.label("Path");
            });
            h.col(|ui| {
                ui.label("Value(avg)");
            });
        })
        .body(|mut body| {
            for d in diagnostics.iter() {
                body.row(32., |mut row| {
                    row.col(|ui| {
                        ui.label(d.path().as_str());
                    });
                    row.col(|ui| {
                        ui.label(format!("{:0.2}", d.average().unwrap_or_default()));
                    });
                });
            }
        });
}

fn ui_plot(ui: &mut egui::Ui, diagnostics: &Res<'_, DiagnosticsStore>) {
    #[cfg(feature = "trace")]
    let _span = info_span!("debug/ui", name = "ui_plot").entered();
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
}

fn render_fps_graph(plt_ui: &mut egui_plot::PlotUi, fps: &Diagnostic) {
    let _span = info_span!("debug/ui", name = "render_fps_graph").entered();
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
