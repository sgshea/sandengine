use bevy::{prelude::*, render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}}};

use crate::screen::Screen;

use super::{world::PixelWorld, LoadedChunks};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedPostUpdate, (create_chunk_displays, update_chunk_displays).run_if(in_state(Screen::Playing)));
}

#[derive(Component)]
struct ChunkDisplayComponent {
    pub chunk: IVec2,
}

/// Creates the chunk textures for each chunk
/// Can do in runtime (so that we can load/unload chunks later)
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
                        ((pos.as_vec2() + 0.5) * pxl_sim.chunk_size.as_vec2()).extend(0.0)
                    ),
                    sprite: Sprite {
                        flip_y: true,
                        ..default()
                    },
                    ..default()
                },
                ChunkDisplayComponent { chunk: *pos },
            ));
            loaded.chunks.push(*pos);
        }
    }
}

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