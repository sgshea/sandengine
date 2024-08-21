pub mod world;
mod chunk;
mod chunk_handler;
mod geometry_helpers;
mod display;
pub mod cell;
pub mod debug;
pub mod interaction;

use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{pixel::world::PixelWorld, rigid::SandEngineRigidPlugin, MainCamera, CHUNKS, WORLD_SIZE};

pub struct PixelPlugin;

impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LoadedChunks::default())
            .add_systems(Startup, setup_pixel_simulation)
            .add_systems(
                FixedUpdate,
                (update_pixel_simulation)
                .chain()
            )
            .add_plugins(display::plugin)
            .add_plugins(interaction::plugin)
            .add_plugins(SandEngineRigidPlugin);

        app.add_plugins(debug::plugin);
    }
}

#[derive(Component)]
pub(crate) struct PixelSimulation {
    pub world: PixelWorld,
}

#[derive(Resource, Default)]
pub(crate) struct LoadedChunks {
    pub chunks: Vec<IVec2>,
}

fn setup_pixel_simulation(
    mut commands: Commands,
    ) {
    commands.spawn((Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: WORLD_SIZE.x as f32,
                min_height: WORLD_SIZE.y as f32,
            },
            near: -1000.0,
            ..default()
        },
        transform: Transform::from_xyz(WORLD_SIZE.x as f32 / 2.0, WORLD_SIZE.y as f32 / 2.0, 1000.0),
        ..default()
    }, MainCamera));

    let world = PixelWorld::new(WORLD_SIZE.x, WORLD_SIZE.y, CHUNKS.x, CHUNKS.y);

    commands.spawn((
        PixelSimulation {
            world,
        },
    ));
}

pub fn update_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
) {
    query.single_mut().world.update();
}