//! Debug egui window for information directly relating to pixel world

use bevy::color::palettes::css::{LIGHT_GRAY, LIGHT_GREEN};
use bevy::prelude::*;

use bevy::math::IVec2;
use bevy_egui::{egui, EguiContexts};

use crate::{CHUNK_SIZE, WORLD_SIZE};

use super::cell::Cell;
use super::interaction::PixelInteraction;
use super::world::PixelWorld;
use super::PixelSimulation;

#[derive(Resource, Default)]
struct PixelSimulationDebug {
    // Hovered position in chunk coordinates
    pub position_in_chunk: IVec2,
    // Currently hovered cell
    pub hovered_cell: Option<Cell>,
    // Is cursor inside the chunk's dirty rect
    pub inside_dirty_rect: bool,
    // Position of hovered chunk
    pub chunk_position: IVec2,
    // Amount of chunks
    pub chunk_amount: u32,
    // Size of chunks
    pub chunk_size: u32,

    pub show_chunk_borders: bool,

    
}

pub(super) fn plugin(app: &mut App) {
        app.add_systems(FixedUpdate, pixel_simulation_debug);

        app.init_resource::<PixelSimulationDebug>();
        app.add_systems(Update, pixel_simulation_debug_ui);

        app.init_gizmo_group::<ChunkGizmos>();
        app.add_systems(PostUpdate, (draw_chunk_gizmos, update_pixel_debug_gizmos));
}

fn pixel_simulation_debug(
    sim: Query<&mut PixelSimulation>,
    mut dbg: ResMut<PixelSimulationDebug>,
    pxl: Res<PixelInteraction>,
) {
    let world = &sim.single().world;

    let cell_pos = pxl.hovered_position;
    dbg.position_in_chunk = PixelWorld::cell_to_position_in_chunk(cell_pos);
    dbg.chunk_position = PixelWorld::cell_to_chunk_position(cell_pos);
    dbg.hovered_cell = world.get_cell(cell_pos);
    dbg.inside_dirty_rect = world.cell_inside_dirty(cell_pos);

    dbg.chunk_size = world.get_chunk_width() as u32;
    dbg.chunk_amount = world.get_chunks().len() as u32;
}

fn pixel_simulation_debug_ui(
    mut ctx: EguiContexts,
    mut dbg: ResMut<PixelSimulationDebug>,
    pxl: Res<PixelInteraction>,
) {
    egui::Window::new("Pixel Debug").show(ctx.ctx_mut(),
        | ui | {
            ui.set_min_width(200.);
            ui.label(format!("Debug info for pixel sim"));
            ui.separator();
            ui.label(format!("Current Chunk: {:?}", dbg.chunk_position));
            ui.label(format!("Current Cell: {:?}", dbg.hovered_cell));
            ui.label(format!("Inside dirty rect?: {:?}", dbg.inside_dirty_rect));
            ui.label(format!("Cell position in world: {:?}", pxl.hovered_position));
            ui.label(format!("Cell position in chunk: {:?}", dbg.position_in_chunk));
            ui.separator();
            ui.label(format!("Amount of chunks/chunk size: {:?}/{:?}", dbg.chunk_amount, dbg.chunk_size));
            ui.checkbox(&mut dbg.show_chunk_borders, "Show Chunks (F2)");
        }
    );
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ChunkGizmos {}

pub fn draw_chunk_gizmos(
    mut chunk_gizmos: Gizmos<ChunkGizmos>,
    pixel_query: Query<&PixelSimulation>,
) {
    let origin_x = WORLD_SIZE.x as f32 / 2.0;
    let origin_y = WORLD_SIZE.y as f32 / 2.0;

    let sim = pixel_query.single();

    let awake_chunks = sim.world.get_chunk_dirty_rects();

    for (pos, rect) in awake_chunks {
        // Calculate position in screen
        let pos_x = (pos.x as f32 * CHUNK_SIZE.x as f32) - WORLD_SIZE.x as f32 / 2.0;
        let pos_y = (pos.y as f32 * CHUNK_SIZE.y as f32) - WORLD_SIZE.y as f32 / 2.0;
        
        // Draw light gray outline of chunk
        chunk_gizmos.rect_2d(
            Vec2::new(
                origin_x + pos_x + CHUNK_SIZE.x as f32 / 2.,
                origin_y + pos_y + CHUNK_SIZE.y as f32 / 2.,
            ),
            0.0,
            CHUNK_SIZE.as_vec2(),
            LIGHT_GRAY,
        );
        // Draw green outline of dirty rect if exists
        if !rect.is_empty() {
            chunk_gizmos.rect_2d(
                Vec2::new(
                    origin_x + pos_x,
                    origin_y + pos_y,
                ) + rect.center_display(),
                0.0,
                rect.size().as_vec2(),
                LIGHT_GREEN,
            );
        }
    }
}

fn update_pixel_debug_gizmos(
    mut config_store: ResMut<GizmoConfigStore>,
    mut dbg: ResMut<PixelSimulationDebug>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let (chunk_config, _) = config_store.config_mut::<ChunkGizmos>();

    chunk_config.enabled = dbg.show_chunk_borders;

    if keyboard.just_pressed(KeyCode::F2) {
        chunk_config.enabled ^= true;
        dbg.show_chunk_borders = chunk_config.enabled;
    }
}