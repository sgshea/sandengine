pub mod world;
mod chunk;
mod chunk_handler;
mod geometry_helpers;
pub mod cell;
pub mod debug;
pub mod interaction;

use bevy::{prelude::*, render::{camera::ScalingMode, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::ImageSampler}};
use bevy_mod_picking::prelude::*;

use crate::{pixel::world::PixelWorld, rigid::SandEngineRigidPlugin, MainCamera, WindowInformation, CHUNKS, RESOLUTION, WORLD_SIZE};

pub struct PixelPlugin;

impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SandEngineRigidPlugin)
            .add_systems(Startup, setup_pixel_simulation)
            .add_systems(
                FixedUpdate,
                (update_pixel_simulation, render_pixel_simulation)
                .chain()
            )
            .add_plugins(interaction::plugin);

        #[cfg(feature = "dev")]
        app.add_plugins(debug::plugin);
    }
}

#[derive(Component)]
pub(crate) struct PixelSimulation {
    pub world: PixelWorld,
    pub image_handle: Handle<Image>,
}

fn setup_pixel_simulation(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut window_info: ResMut<WindowInformation>,
    ) {
    commands.spawn((Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WORLD_SIZE.x as f32,
                height: WORLD_SIZE.y as f32,
            },
            near: -1000.0,
            ..default()
        },
        transform: Transform::from_xyz(WORLD_SIZE.x as f32 / 2.0, WORLD_SIZE.y as f32 / 2.0, 1000.0),
        ..default()
    }, MainCamera));

    window_info.scale = (RESOLUTION.x / WORLD_SIZE.x as f32, RESOLUTION.y / WORLD_SIZE.y as f32);

    let world = PixelWorld::new(WORLD_SIZE.x, WORLD_SIZE.y, CHUNKS.x, CHUNKS.y);

    let mut image = Image::new(
        Extent3d {
            width: WORLD_SIZE.x as u32,
            height: WORLD_SIZE.y as u32,
            ..default()
        },
        TextureDimension::D2,
        vec![0; (WORLD_SIZE.x * WORLD_SIZE.y * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    let image_handle = images.add(image);

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(WORLD_SIZE.x as f32 / 2.0, WORLD_SIZE.y as f32 / 2.0, 0.0),
            ..default()
        },
        PixelSimulation {
            world,
            image_handle: image_handle.clone(),
        },
    ))
    .with_children(|children| {
            children.spawn((
                Name::new("Image"),
                SpriteBundle {
                    texture: image_handle,
                    sprite: Sprite {
                        flip_y: true,
                        ..default()
                    },
                    ..default()
                },
                PickableBundle::default(),
            ));
        });
}

fn update_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
) {
    query.single_mut().world.update();
}

// Simple rendering function which iterates over all cells and copies to single image
// Potential improvement later is to do this per chunk and only in updating chunks
fn render_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
    mut images: ResMut<Assets<Image>>,
) {
    for sim in query.iter_mut() {
        let image = images.get_mut(&sim.image_handle).unwrap();
        image.data.chunks_mut(4).enumerate().for_each(|(i, pixel)| {
            let x = i as i32 % WORLD_SIZE.x;
            let y = i as i32 / WORLD_SIZE.x;
            let cell = sim.world.get_cell(IVec2 { x, y }).expect("Cell out of bounds");
            let color = &cell.color;
            pixel.copy_from_slice(color);
        });
    }
}