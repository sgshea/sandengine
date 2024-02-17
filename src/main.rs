mod cell;
mod cell_types;
mod world;
mod chunk;

use std::time;

use bevy::{prelude::*, render::{camera::ScalingMode, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::ImageSampler}, window::PresentMode};
use bevy_mod_picking::{backends::egui::bevy_egui, prelude::*};
// bevy_egui re-exported from bevy_mod_picking
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use cell_types::CellType;

const RESOLUTION: (f32, f32) = (1920.0, 1080.0);
const WORLD_SIZE: (i32, i32) = (512, 512);
const SCALE: (f32, f32) = (RESOLUTION.0 / WORLD_SIZE.0 as f32, RESOLUTION.1 / WORLD_SIZE.1 as f32);
const CHUNKS: (i32, i32) = (8, 8);
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
        .init_resource::<PixelSimulationInteraction>()
        .add_systems(Startup, setup_pixel_simulation)
        .add_systems(FixedUpdate, update_pixel_simulation)
        .add_systems(PostUpdate, render_pixel_simulation)
        .add_systems(Update, egui_ui)
        .add_systems(Update, cell_selector_ui)
        .run();
}

#[derive(Component)]
pub struct PixelSimulation {
    pub world: world::PixelWorld,
    pub image_handle: Handle<Image>,
}

#[derive(Resource)]
struct PixelSimulationInteraction {
    pub selected_cell: CellType,
}

impl Default for PixelSimulationInteraction {
    fn default() -> Self {
        PixelSimulationInteraction {
            selected_cell: CellType::Sand,
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource, Default)]
struct DebugInfo {
    pub sim_time: f32,
    pub render_construct_time: f32,
    pub position: Vec2,
    pub chunk_position: Vec2,
    pub cell_position_in_chunk: Vec2,
    pub hovered_cell: Option<cell::Cell>,
}

fn place_cells_at_pos(
    mut sim: Query<&mut PixelSimulation>,
    pos: Vec2,
    cell_type: cell_types::CellType,
) {
    for sim in sim.iter_mut() {
        for x in -5..5 {
            for y in -5..5 {
                sim.world.set_cell(pos.x as i32 + x, pos.y as i32 + y, cell::Cell::cell_from_type(cell_type));
            }
        }
    }
}

fn cell_at_pos_dbg(
    sim: Query<&mut PixelSimulation>,
    pos: Vec2,
    mut dbg_info: ResMut<DebugInfo>,
) {
    for sim in sim.iter() {
        dbg_info.position = pos;
        dbg_info.chunk_position = Vec2::new((pos.x / CHUNK_SIZE.0 as f32).floor(), (pos.y / CHUNK_SIZE.1 as f32).floor());
        dbg_info.cell_position_in_chunk = Vec2::new((pos.x % CHUNK_SIZE.0 as f32).floor(), (pos.y % CHUNK_SIZE.1 as f32).floor());
        dbg_info.hovered_cell = Some(sim.world.get_cell(pos.x as i32, pos.y as i32));
    }
}

fn setup_pixel_simulation(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    ) {
    commands.spawn((Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: RESOLUTION.0,
                height: RESOLUTION.1,
            },
            near: -1000.0,
            ..default()
        },
        ..default()
    }, MainCamera));

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
                    transform: Transform {
                        scale: Vec3::new(SCALE.0, SCALE.1, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        flip_y: true,
                        ..default()
                    },
                    ..default()
                },
                PickableBundle::default(),
                On::<Pointer<Click>>::run(|event: Listener<Pointer<Click>>, sim: Query<&mut PixelSimulation>, pixel_interaction: ResMut<PixelSimulationInteraction>| {
                    if event.button == PointerButton::Primary {
                        let event_pos = event.pointer_location.position;
                        let cell_position = Vec2::new(
                            event_pos.x / SCALE.0,
                            WORLD_SIZE.1 as f32 - (event_pos.y / SCALE.1),
                        );
                        place_cells_at_pos(sim, cell_position, pixel_interaction.selected_cell);
                    }
                }),
                On::<Pointer<Drag>>::run(|event: Listener<Pointer<Drag>>, sim: Query<&mut PixelSimulation>, pixel_interaction: ResMut<PixelSimulationInteraction>| {
                    if event.button == PointerButton::Primary {
                        let event_pos = event.pointer_location.position;
                        let cell_position = Vec2::new(
                            event_pos.x / SCALE.0,
                            WORLD_SIZE.1 as f32 - (event_pos.y / SCALE.1),
                        );
                        place_cells_at_pos(sim, cell_position, pixel_interaction.selected_cell);
                    }
                }),
                On::<Pointer<Move>>::run(|event: Listener<Pointer<Move>>, sim: Query<&mut PixelSimulation>, dbg_info: ResMut<DebugInfo>| {
                    let event_pos = event.pointer_location.position;
                    let cell_position = Vec2::new(
                        event_pos.x / SCALE.0,
                        WORLD_SIZE.1 as f32 - (event_pos.y / SCALE.1),
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
    dbg_info.sim_time = start.elapsed().as_secs_f32();
}

fn render_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
    mut images: ResMut<Assets<Image>>,
    mut dbg_info: ResMut<DebugInfo>,
) {
    let start = time::Instant::now();
    for sim in query.iter_mut() {
        let image = images.get_mut(&sim.image_handle).unwrap();
        for y in 0..WORLD_SIZE.1 as usize {
            for x in 0..WORLD_SIZE.0 as usize {
                let cell = sim.world.get_cell(x as i32, y as i32);
                let cell_color = cell.get_cell_color();
                let index = (x + y * WORLD_SIZE.0 as usize) * 4;
                image.data[index] = (cell_color[0] * 255.0) as u8;
                image.data[index + 1] = (cell_color[1] * 255.0) as u8;
                image.data[index + 2] = (cell_color[2]* 255.0) as u8;
                image.data[index + 3] = 255;
            }
        }
    }
    dbg_info.render_construct_time = start.elapsed().as_secs_f32();
}

fn egui_ui(
    mut ctx: EguiContexts,
    dbg_info: ResMut<DebugInfo>,
) {
    egui::Window::new("Debug Info")
    .show(ctx.ctx_mut(),
        |ui| {
            ui.set_min_width(200.0);
            // convert to ms
            let sim_t_ms = dbg_info.sim_time * 1000.0;
            let render_construct_t_ms = dbg_info.render_construct_time * 1000.0;
            ui.label(format!("Sim Time: {:.2}ms", sim_t_ms));
            ui.label(format!("Render Construct Time: {:.2}ms", render_construct_t_ms));
            ui.label(format!("FPS: {:.2}", 1.0 / dbg_info.sim_time));
            ui.label(format!("Position: {:?}", dbg_info.position));
            ui.label(format!("Chunk Position: {:?}", dbg_info.chunk_position));
            ui.label(format!("Cell Position in Chunk: {:?}", dbg_info.cell_position_in_chunk));
            ui.label(format!("Hovered Cell: {:?}", dbg_info.hovered_cell));
        }
    );
}

fn cell_selector_ui(
    mut ctx: EguiContexts,
    mut pixel_interaction: ResMut<PixelSimulationInteraction>,
) {
    egui::Window::new("Cell Selector")
    .show(ctx.ctx_mut(),
        |ui| {
            ui.set_min_width(100.0);
            ui.radio_value(&mut pixel_interaction.selected_cell, CellType::Sand, "Sand");
            ui.radio_value(&mut pixel_interaction.selected_cell, CellType::Water, "Water");
            ui.radio_value(&mut pixel_interaction.selected_cell, CellType::Stone, "Stone");
            ui.radio_value(&mut pixel_interaction.selected_cell, CellType::Empty, "Empty");
        }
    );
}
