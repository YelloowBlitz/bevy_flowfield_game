mod debug;

use bevy::prelude::*;
use bevy::render::options::{Backends, WgpuOptions};

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

fn main() {
    App::new()
        .insert_resource(WgpuOptions {
            backends: Some(Backends::DX12),
            ..Default::default()
        })
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            title: "My Game".to_string(),
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(debug::DebugPlugin)
        .run();
}
