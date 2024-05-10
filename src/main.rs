mod cell;
mod cell_types;
mod world;
mod chunk;
mod cworker;

mod debug_ui;

use std::time;

use bevy::{prelude::*, render::{camera::ScalingMode, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::ImageSampler}, window::{PresentMode, WindowResized}};
use bevy_mod_picking::{backends::egui::bevy_egui, prelude::*};
// bevy_egui re-exported from bevy_mod_picking
use bevy_egui::EguiPlugin;
use debug_ui::{cell_at_pos_dbg, cell_selector_ui, draw_chunk_gizmos, egui_ui, place_cells_at_pos, update_gizmos_config, ChunkGizmos, DebugInfo, PixelSimulationInteraction};
use rayon::prelude::*;


const RESOLUTION: (f32, f32) = (1920.0, 1080.0);
const WORLD_SIZE: (i32, i32) = (256, 256);
const CHUNKS: (i32, i32) = (4, 4);
const CHUNK_SIZE: (i32, i32) = (WORLD_SIZE.0 / CHUNKS.0, WORLD_SIZE.1 / CHUNKS.1);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Simulation".to_string(),
                    resolution: RESOLUTION.into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()}),
            DefaultPickingPlugins,
            EguiPlugin
        ))
        .init_resource::<DebugInfo>()
        .init_resource::<WindowInformation>()
        .init_gizmo_group::<ChunkGizmos>()
        .init_resource::<PixelSimulationInteraction>()
        .add_systems(Startup, setup_pixel_simulation)
        .add_systems(FixedUpdate, update_pixel_simulation.run_if(in_state(AppState::Running)))
        .add_systems(PostUpdate, render_pixel_simulation.run_if(in_state(AppState::Running)))
        .add_systems(Update, egui_ui)
        .add_systems(Update, cell_selector_ui)
        .add_systems(Update, resize_window)
        .add_systems(PostUpdate, (draw_chunk_gizmos, update_gizmos_config))
        .init_state::<AppState>()
        .run();
}

#[derive(Component)]
pub struct PixelSimulation {
    pub world: world::PixelWorld,
    pub image_handle: Handle<Image>,
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum AppState {
    #[default]
    Running,
    Paused,
}

#[derive(Resource, Default)]
struct WindowInformation {
    scale: (f32, f32),
}

#[derive(Component)]
struct MainCamera;

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

    let world = world::PixelWorld::new(WORLD_SIZE.0, WORLD_SIZE.1, CHUNKS.0, CHUNKS.1);

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
    query.iter_mut().next().unwrap().world.update();
    let elapsed = start.elapsed().as_secs_f32();
    dbg_info.sim_time.push(elapsed);
    if dbg_info.sim_time.len() > 100 {
        dbg_info.sim_time.remove(0);
    }
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
        image.data.par_chunks_mut(4).enumerate().for_each(|(i, pixel)| {
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

fn resize_window(
    mut events: EventReader<WindowResized>,
    mut window_info: ResMut<WindowInformation>,
) {
    match events.read().last() {
        Some(event) => {
            window_info.scale = (event.width / WORLD_SIZE.0 as f32, event.height / WORLD_SIZE.1 as f32);
        },
        None => {}
    }
}
