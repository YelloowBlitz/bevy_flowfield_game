use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::{App, Plugin};
// use bevy_inspector_egui::WorldInspectorPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(LogDiagnosticsPlugin::default());
            app.add_plugin(FrameTimeDiagnosticsPlugin::default());
            // app.add_plugin(WorldInspectorPlugin::new());
        }
    }
}
