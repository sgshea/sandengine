use std::time;

use bevy::{prelude::*, render::{camera::ScalingMode, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::ImageSampler}};
use bevy_mod_picking::prelude::*;

use crate::{debug_ui::{cell_at_pos_dbg, draw_chunk_gizmos, place_cells_at_pos, update_gizmos_config, DebugInfo, PixelSimulationInteraction}, rigid::SandEngineRigidPlugin, world::PixelWorld, AppState, MainCamera, WindowInformation, CHUNKS, RESOLUTION, WORLD_SIZE};

pub struct PixelPlugin;
impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PixelSimulationInteraction>()
            .add_plugins(SandEngineRigidPlugin)
            .add_systems(Startup, setup_pixel_simulation)
            .add_systems(
                FixedUpdate,
                (update_pixel_simulation, render_pixel_simulation)
                .chain()
                .distributive_run_if(in_state(AppState::Running)),
            )
            .add_systems(PostUpdate, (draw_chunk_gizmos, update_gizmos_config));

    }
}

#[derive(Component)]
pub struct PixelSimulation {
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
                width: WORLD_SIZE.0 as f32,
                height: WORLD_SIZE.1 as f32,
            },
            near: -1000.0,
            ..default()
        },
        ..default()
    }, MainCamera));

    window_info.scale = (RESOLUTION.0 / WORLD_SIZE.0 as f32, RESOLUTION.1 / WORLD_SIZE.1 as f32);

    let world = PixelWorld::new(WORLD_SIZE.0, WORLD_SIZE.1, CHUNKS.0, CHUNKS.1);

    let mut image = Image::new(
        Extent3d {
            width: WORLD_SIZE.0 as u32,
            height: WORLD_SIZE.1 as u32,
            ..default()
        },
        TextureDimension::D2,
        vec![0; (WORLD_SIZE.0 * WORLD_SIZE.1 * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    let image_handle = images.add(image);

    // Image does not fill entire screen
    // Will refactor/handle later once we have chunks as may decide to render each chunk separately (attaching image to each)
    commands.spawn((
        SpatialBundle::default(),
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
                On::<Pointer<Click>>::run(|event: Listener<Pointer<Click>>, sim: Query<&mut PixelSimulation>, pixel_interaction: ResMut<PixelSimulationInteraction>, window_info: ResMut<WindowInformation>| {
                    if event.button == PointerButton::Primary {
                        let event_pos = event.pointer_location.position;
                        let cell_position = Vec2::new(
                            event_pos.x / window_info.scale.0,
                            WORLD_SIZE.1 as f32 - (event_pos.y / window_info.scale.1),
                        );
                        place_cells_at_pos(sim, pixel_interaction.cell_amount, cell_position, pixel_interaction.selected_cell);
                    }
                }),
                On::<Pointer<Drag>>::run(|event: Listener<Pointer<Drag>>, sim: Query<&mut PixelSimulation>, pixel_interaction: ResMut<PixelSimulationInteraction>, window_info: ResMut<WindowInformation>| {
                    if event.button == PointerButton::Primary {
                        let event_pos = event.pointer_location.position;
                        let cell_position = Vec2::new(
                            event_pos.x / window_info.scale.0,
                            WORLD_SIZE.1 as f32 - (event_pos.y / window_info.scale.1),
                        );
                        place_cells_at_pos(sim, pixel_interaction.cell_amount, cell_position, pixel_interaction.selected_cell);
                    }
                }),
                On::<Pointer<Move>>::run(|event: Listener<Pointer<Move>>, sim: Query<&mut PixelSimulation>, dbg_info: ResMut<DebugInfo>, window_info: ResMut<WindowInformation> | {
                    let event_pos = event.pointer_location.position;
                    let cell_position = Vec2::new(
                        event_pos.x / window_info.scale.0,
                        WORLD_SIZE.1 as f32 - (event_pos.y / window_info.scale.1),
                    );
                    if cell_position.x < 0. || cell_position.y < 0. || cell_position.x > WORLD_SIZE.0 as f32 || cell_position.y > WORLD_SIZE.1 as f32 {
                        // these are invalid
                        return;
                    }
                    cell_at_pos_dbg(sim, cell_position, dbg_info);
                }),
            ));
        });
}

fn update_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
    mut dbg_info: ResMut<DebugInfo>,
) {
    let start = time::Instant::now();
    query.single_mut().world.update();
    let elapsed = start.elapsed().as_secs_f32();
    dbg_info.sim_time.push(elapsed);
    if dbg_info.sim_time.len() > 100 {
        dbg_info.sim_time.remove(0);
    }
}

fn render_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
    mut images: ResMut<Assets<Image>>,
    mut dbg_info: ResMut<DebugInfo>,
) {
    let start = time::Instant::now();
    for sim in query.iter_mut() {
        let image = images.get_mut(&sim.image_handle).unwrap();
        image.data.chunks_mut(4).enumerate().for_each(|(i, pixel)| {
            let x = i as i32 % WORLD_SIZE.0;
            let y = i as i32 / WORLD_SIZE.0;
            let cell = sim.world.get_cell(x, y).expect("Cell out of bounds");
            let color = cell.get_color();
            pixel.copy_from_slice(color);
        });
    }
    let elapsed = start.elapsed().as_secs_f32();
    dbg_info.render_construct_time.push(elapsed);
    if dbg_info.render_construct_time.len() > 100 {
        dbg_info.render_construct_time.remove(0);
    }
}
