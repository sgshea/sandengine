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
        .init_state::<WorldSizes>()
        .insert_resource(Time::<Fixed>::from_hz(64.))
        .add_plugins(input::plugin)
        .add_plugins((ui::plugin, screen::plugin))
        .add_plugins(PixelPlugin)
        .add_plugins(SandEngineRigidPlugin)
        .add_plugins(ParticlePlugin);
    }
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

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default, Copy, Reflect)]
pub enum WorldSizes {
    Small,
    #[default]
    Medium,
    Large,
}

pub fn spawn_worlds(commands: &mut Commands, world_size: Res<State<WorldSizes>>) {
    let (world_size, chunk_amount) = match *world_size.get() {
        WorldSizes::Small => (UVec2::new(128, 128), UVec2::new(2, 2)),
        WorldSizes::Medium => (UVec2::new(256, 256), UVec2::new(4, 4)),
        WorldSizes::Large => (UVec2::new(512, 512), UVec2::new(8, 8)),
    };
    commands.add(SpawnWorlds {
        world_size,
        chunk_amount
    });
}