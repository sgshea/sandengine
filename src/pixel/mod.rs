pub mod world;
mod chunk;
mod chunk_handler;
mod geometry_helpers;
mod display;
pub mod cell;
pub mod debug;
pub mod interaction;

use bevy::{prelude::*, render::{camera::ScalingMode, view::RenderLayers}};
use display::setup_gradient_background;

use crate::{pixel::world::PixelWorld, screen::Screen, SpawnWorlds};

pub struct PixelPlugin;

impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LoadedChunks::default())
            .add_systems(
                FixedUpdate,
                update_pixel_simulation
                .run_if(in_state(Screen::Playing))
            )
            .add_plugins((display::plugin, interaction::plugin));

        app.add_plugins(debug::plugin);
    }
}

#[derive(Resource, Default)]
pub(crate) struct LoadedChunks {
    pub chunks: Vec<IVec2>,
}

#[derive(Component)]
pub struct GameCamera;

pub fn spawn_pixel_world(
    In(config): In<SpawnWorlds>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: config.world_size.x as f32,
                min_height: config.world_size.y as f32,
            },
            near: -1000.0,
            ..default()
        },
        transform: Transform::from_translation((config.world_size.as_vec2() / 2.).extend(1000.)),
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    })
    .insert((
        StateScoped(Screen::Playing),
        // Layers: 0 (default), 1 (rigidbodies), 2 (cells/pixels), 3 (particles)
        RenderLayers::from_layers(&[0, 1, 2, 3]),
        GameCamera
    ));

    let world = PixelWorld::new(config.world_size, config.chunk_amount);

    commands.spawn(
        world
    ).insert(StateScoped(Screen::Playing));

    setup_gradient_background(&mut commands, meshes, materials, &config);

    // Reset loaded chunks
    loaded_chunks.chunks.clear();
}

pub fn update_pixel_simulation(
    mut query: Query<&mut PixelWorld>,
) {
    query.single_mut().update();
}