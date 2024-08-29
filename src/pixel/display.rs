use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::RenderLayers,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{screen::Screen, SpawnWorlds};

use super::{world::PixelWorld, LoadedChunks};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedPostUpdate,
        (create_chunk_displays, update_chunk_displays).run_if(in_state(Screen::Playing)),
    );
}

// Component used in a bundle with the corresponding display image of a chunk
#[derive(Component)]
struct ChunkDisplayComponent {
    pub chunk: IVec2,
}

// Creates the chunk textures for each chunk
// Can do in runtime (so that we can load/unload chunks later)
fn create_chunk_displays(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    pxl_sim: Query<&PixelWorld>,
    mut loaded: ResMut<LoadedChunks>,
) {
    let pxl_sim = &pxl_sim.single();

    // Find all chunks that do not have an image and create one
    for (pos, _chunk) in &pxl_sim.chunks {
        if !loaded.chunks.contains(pos) {
            let image = Image::new(
                Extent3d {
                    width: pxl_sim.get_chunk_width(),
                    height: pxl_sim.get_chunk_height(),
                    ..default()
                },
                TextureDimension::D2,
                vec![0; (pxl_sim.get_chunk_width() * pxl_sim.get_chunk_height() * 4) as usize],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            commands.spawn((
                SpriteBundle {
                    texture: images.add(image),
                    transform: Transform::from_translation(
                        ((pos.as_vec2() + 0.5) * pxl_sim.chunk_size.as_vec2()).extend(2.),
                    ),
                    sprite: Sprite {
                        flip_y: true,
                        ..default()
                    },
                    ..default()
                },
                ChunkDisplayComponent { chunk: *pos },
                StateScoped(Screen::Playing),
                RenderLayers::layer(2),
            ));
            loaded.chunks.push(*pos);
        }
    }
}

// Updates all chunk displays if they have updated
fn update_chunk_displays(
    pxl_sim: Query<&PixelWorld>,
    mut chunks_display: Query<(&ChunkDisplayComponent, &mut Handle<Image>)>,
    mut images: ResMut<Assets<Image>>,
) {
    let pxl_sim = &pxl_sim.single();

    for (chunk_display, handle) in chunks_display.iter_mut() {
        if let Some(data) = pxl_sim.should_render_data(chunk_display.chunk) {
            let current = images.get_mut(&handle.clone()).unwrap();
            current.data = data;
        }
    }
}

// Create a gradient background to be displayed behind the world
pub fn setup_gradient_background(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: &SpawnWorlds,
) {
    // Build a default quad mesh
    let mut mesh = Mesh::from(Rectangle::default());
    // Build vertex colors for the quad. One entry per vertex (the corners of the quad)
    let vertex_colors: Vec<[f32; 4]> = vec![
        LinearRgba::BLUE.to_f32_array(),
        LinearRgba::BLUE.to_f32_array(),
        LinearRgba::WHITE.darker(0.1).to_f32_array(),
        LinearRgba::WHITE.darker(0.1).to_f32_array(),
    ];
    // Insert the vertex colors as an attribute
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);

    let mesh_handle: Mesh2dHandle = meshes.add(mesh).into();

    // Spawn the quad with vertex colors
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: mesh_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.))
                .with_scale((config.world_size.as_vec2() * 4.).extend(0.)),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(StateScoped(Screen::Playing));
}
