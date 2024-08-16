mod pixel;
mod rigid;

mod input;

mod dev_tools;
mod states;

use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::prelude::*;
use bevy_egui::EguiPlugin;
use pixel::PixelPlugin;
use states::DebugState;

const WORLD_SIZE: IVec2 = IVec2::new(256, 256);
const CHUNKS: IVec2 = IVec2::new(4, 4);
const CHUNK_SIZE: IVec2 = IVec2::new(WORLD_SIZE.x / CHUNKS.x, WORLD_SIZE.y / CHUNKS.y);

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Simulation".to_string(),
                    present_mode: PresentMode::AutoVsync,
                    canvas: Some("#bevy".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()}
            ).set(ImagePlugin::default_nearest()),
            DefaultPickingPlugins,
            EguiPlugin,
            dev_tools::plugin,
        ))
        .init_state::<DebugState>()
        .insert_resource(Time::<Fixed>::from_hz(64.))
        .add_plugins(input::plugin)
        .add_plugins(PixelPlugin);
    }
}

#[derive(Component)]
struct MainCamera;