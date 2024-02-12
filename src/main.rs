mod cell;
mod cell_types;
mod world;

use std::time;

use bevy::{prelude::*, render::{camera::ScalingMode, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::ImageSampler}, window::{PresentMode, PrimaryWindow}};
use bevy_mod_picking::{backends::egui::bevy_egui, prelude::*};
// bevy_egui re-exported from bevy_mod_picking
use bevy_egui::{egui, EguiContexts, EguiPlugin};

const RESOLUTION : (f32, f32) = (1920.0, 1080.0);
const WORLD_SIZE : (i32, i32) = (1920, 1080);
const SCALE: (f32, f32) = (RESOLUTION.0 / WORLD_SIZE.0 as f32, RESOLUTION.1 / WORLD_SIZE.1 as f32);

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
        .add_systems(Startup, setup_pixel_simulation)
        .add_systems(FixedUpdate, update_pixel_simulation)
        .add_systems(PostUpdate, render_pixel_simulation)
        .add_systems(Update, egui_ui)
        .run();
}

#[derive(Component)]
pub struct PixelSimulation {
    pub world: world::PixelWorld,
    pub image_handle: Handle<Image>,
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource, Default)]
struct DebugInfo {
    pub sim_time: f32,
    pub render_construct_time: f32,
}

fn place_cells_at_pos(
    mut sim: Query<&mut PixelSimulation>,
    pos: Vec2,
    cell_type: cell_types::CellType,
) {
    for mut sim in sim.iter_mut() {
        for x in -10..10 {
            for y in -10..10 {
                sim.world.set_cell_checked(pos.x as i32 + x, pos.y as i32 + y, cell::Cell::cell_from_type(cell_type));
            }
        }
    }
}

fn setup_pixel_simulation(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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

    let world = world::PixelWorld::new(WORLD_SIZE.0, WORLD_SIZE.1);

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
                On::<Pointer<Click>>::run(|event: Listener<Pointer<Click>>, sim: Query<&mut PixelSimulation>| {
                    if event.button == PointerButton::Primary {
                        let event_pos = event.pointer_location.position;
                        let cell_position = Vec2::new(
                            event_pos.x / SCALE.0,
                            WORLD_SIZE.1 as f32 - (event_pos.y / SCALE.1),
                        );
                        place_cells_at_pos(sim, cell_position, cell_types::CellType::Sand);
                    }
                }),
                On::<Pointer<Drag>>::run(|event: Listener<Pointer<Drag>>, sim: Query<&mut PixelSimulation>| {
                    if event.button == PointerButton::Primary {
                        let event_pos = event.pointer_location.position;
                        let cell_position = Vec2::new(
                            event_pos.x / SCALE.0,
                            WORLD_SIZE.1 as f32 - (event_pos.y / SCALE.1),
                        );
                        place_cells_at_pos(sim, cell_position, cell_types::CellType::Sand);
                    }
                }),
            ));
        });
    
}

fn update_pixel_simulation(
    mut query: Query<&mut PixelSimulation>,
    mut dbg_info: ResMut<DebugInfo>,
) {
    let start = time::Instant::now();
    for mut sim in query.iter_mut() {
        sim.world.update();
    }
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
                let cell_color = sim.world.get_cell_2d(x as i32, y as i32).get_cell_color();
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
    egui::Window::new("Debug Info").show(ctx.ctx_mut(),
        |ui| {
            // convert to ms
            let sim_t_ms = dbg_info.sim_time * 1000.0;
            let render_construct_t_ms = dbg_info.render_construct_time * 1000.0;
            ui.label(format!("Sim Time: {:.2}ms", sim_t_ms));
            ui.label(format!("Render Construct Time: {:.2}ms", render_construct_t_ms));
            ui.label(format!("FPS: {:.2}", 1.0 / dbg_info.sim_time));
        }
    );
}