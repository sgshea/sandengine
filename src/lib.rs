mod pixel;
mod particles;
mod rigid;

mod input;

mod dev_tools;
mod states;
pub mod ui;
mod screen;

use bevy::{ecs::{system::RunSystemOnce, world::Command}, prelude::*, window::PresentMode};
use bevy_egui::EguiPlugin;
use particles::ParticlePlugin;
use pixel::{spawn_pixel_world, PixelPlugin};
use rigid::{spawn_rigid_world, SandEngineRigidPlugin};
use screen::Screen;
use states::{AppSet, DebugState};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );
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
        .add_plugins((ui::plugin, screen::plugin))
        .add_plugins(PixelPlugin)
        .add_plugins(SandEngineRigidPlugin)
        .add_plugins(ParticlePlugin);

        app.add_systems(Startup, spawn_camera);
    }
}


fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2dBundle::default(),
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
        StateScoped(Screen::Title)
    ));
}

/// A command to spawn the worlds
#[derive(Debug, Clone, Copy)]
pub struct SpawnWorlds {
    pub world_size: UVec2,
    pub chunk_amount: UVec2,
}

impl Command for SpawnWorlds {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn_pixel_world);

        world.run_system_once_with(self, spawn_rigid_world);
    }
}

pub fn spawn_worlds(world: &mut World) {
    SpawnWorlds {
        world_size: UVec2::new(256, 256),
        chunk_amount: UVec2::new(4, 4),
    }.apply(world);
}