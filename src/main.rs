mod cell;
mod cell_types;
mod world;
mod chunk;
mod cworker;
mod pixel_plugin;

mod rigid;

mod debug_ui;

use bevy::{prelude::*, window::{PresentMode, WindowResized}};
use bevy_mod_picking::{backends::egui::bevy_egui, prelude::*};
// bevy_egui re-exported from bevy_mod_picking
use bevy_egui::EguiPlugin;
use debug_ui::{cell_selector_ui, egui_ui, keyboard_debug, ChunkGizmos, DebugInfo};
use pixel_plugin::PixelPlugin;

const RESOLUTION: (f32, f32) = (1920.0, 1080.0);
const WORLD_SIZE: (i32, i32) = (256, 256);
const CHUNKS: (i32, i32) = (2, 2);
const CHUNK_SIZE: (i32, i32) = (WORLD_SIZE.0 / CHUNKS.0, WORLD_SIZE.1 / CHUNKS.1);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Simulation".to_string(),
                    resolution: RESOLUTION.into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()}),
            DefaultPickingPlugins,
            EguiPlugin
        ))
        .init_resource::<DebugInfo>()
        .init_resource::<WindowInformation>()
        .init_gizmo_group::<ChunkGizmos>()
        .add_systems(Update, egui_ui)
        .add_systems(Update, keyboard_debug)
        .add_systems(Update, cell_selector_ui)
        .add_systems(Update, resize_window)
        .add_plugins(PixelPlugin)
        .init_state::<AppState>()
        .insert_resource(Time::<Fixed>::from_hz(64.))
        .run();
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum AppState {
    #[default]
    Running,
    Paused,
}

#[derive(Resource, Default)]
struct WindowInformation {
    scale: (f32, f32),
}

#[derive(Component)]
struct MainCamera;

fn resize_window(
    mut events: EventReader<WindowResized>,
    mut window_info: ResMut<WindowInformation>,
) {
    match events.read().last() {
        Some(event) => {
            window_info.scale = (event.width / WORLD_SIZE.0 as f32, event.height / WORLD_SIZE.1 as f32);
        },
        None => {}
    }
}
