mod pixel;
mod rigid;

mod input;

mod dev_tools;

use bevy::{prelude::*, window::{PresentMode, WindowResized}};
use bevy_mod_picking::prelude::*;
use bevy_egui::EguiPlugin;
use pixel::PixelPlugin;

const RESOLUTION: Vec2 = Vec2::new(1920.0, 1080.0);
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
                    resolution: RESOLUTION.into(),
                    present_mode: PresentMode::AutoVsync,
                    canvas: Some("#bevy".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()}
            ).set(ImagePlugin::default_nearest()),
            DefaultPickingPlugins,
            EguiPlugin
        ))
        .init_resource::<WindowInformation>()
        .add_systems(Update, resize_window)
        .add_plugins(PixelPlugin)
        .insert_resource(Time::<Fixed>::from_hz(64.));

        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
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
    if let Some(event) = events.read().last() {
        window_info.scale = (event.width / WORLD_SIZE.x as f32, event.height / WORLD_SIZE.y as f32);
    }
}