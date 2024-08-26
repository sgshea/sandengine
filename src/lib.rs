mod pixel;
mod particles;
mod rigid;

mod input;

mod dev_tools;
mod states;

use bevy::{prelude::*, window::PresentMode};
use bevy_egui::EguiPlugin;
use particles::ParticlePlugin;
use pixel::PixelPlugin;
use states::DebugState;

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
            EguiPlugin,
            dev_tools::plugin,
        ))
        .init_state::<DebugState>()
        .insert_resource(Time::<Fixed>::from_hz(64.))
        .add_plugins(input::plugin)
        .add_plugins(PixelPlugin)
        .add_plugins(ParticlePlugin);
    }
}

#[derive(Component)]
struct MainCamera;