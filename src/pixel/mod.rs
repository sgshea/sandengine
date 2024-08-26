pub mod world;
mod chunk;
mod chunk_handler;
mod geometry_helpers;
mod display;
pub mod cell;
pub mod debug;
pub mod interaction;

use bevy::{prelude::*, render::camera::ScalingMode};

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
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 256.,
                min_height: 256.,
            },
            near: -1000.0,
            ..default()
        },
        transform: Transform::from_xyz(256. / 2.0, 256. / 2.0, 1000.0),
        ..default()
    }).insert((StateScoped(Screen::Playing), GameCamera));

    let world = PixelWorld::new(UVec2 { x: 256, y: 256 }, UVec2 { x: 4, y: 4 });

    commands.spawn(
        world
    ).insert(StateScoped(Screen::Playing));
}

pub fn update_pixel_simulation(
    mut query: Query<&mut PixelWorld>,
) {
    query.single_mut().update();
}