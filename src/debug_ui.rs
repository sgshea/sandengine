use bevy::prelude::*;
use bevy_mod_picking::backends::egui::bevy_egui;
// bevy_egui re-exported from bevy_mod_picking
use bevy_egui::{egui, EguiContexts};
use strum::{IntoEnumIterator, VariantNames};

use crate::{cell::Cell, cell_types::CellType, pixel_plugin::PixelSimulation, AppState, CHUNK_SIZE, WORLD_SIZE};

#[derive(Resource)]
pub struct PixelSimulationInteraction {
    pub selected_cell: CellType,
    // How much cells to place when clicking
    pub cell_amount: i32,
}

impl Default for PixelSimulationInteraction {
    fn default() -> Self {
        PixelSimulationInteraction {
            selected_cell: CellType::Sand,
            cell_amount: 12,
        }
    }
}

pub fn place_cells_at_pos(
    mut sim: Query<&mut PixelSimulation>,
    amt_to_place: i32,
    pos: Vec2,
    cell_type: CellType,
) {
    let amt_to_place_quarter = amt_to_place / 4;
    let amt_to_place_half = amt_to_place / 2;
    for mut sim in sim.iter_mut() {
        for x in -amt_to_place_half..=amt_to_place_half {
            for y in -amt_to_place_half..amt_to_place_half {
                // Make circle
                if (x * x) + (y * y) > amt_to_place_quarter * amt_to_place_quarter {
                    continue;
                }
                sim.world.set_cell(pos.x as i32 + x, pos.y as i32 + y, Cell::from(cell_type));
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct DebugInfo {
    pub sim_time: Vec<f32>,
    pub render_construct_time: Vec<f32>,
    pub position: Vec2,
    pub chunk_position: Vec2,
    pub cell_position_in_chunk: Vec2,
    pub hovered_cell: Option<Cell>,

    pub is_paused: bool,

    pub show_gizmos: bool,
}

impl DebugInfo {
    pub fn average_frame_time(&self) -> f32 {
        let sim_time: f32 = self.sim_time.iter().sum();
        (sim_time) / (self.sim_time.len() as f32)
    }

    pub fn average_render_construct_time(&self) -> f32 {
        let render_construct_time: f32 = self.render_construct_time.iter().sum();
        (render_construct_time) / (self.render_construct_time.len() as f32)
    }
}

pub fn cell_at_pos_dbg(
    sim: Query<&mut PixelSimulation>,
    pos: Vec2,
    mut dbg_info: ResMut<DebugInfo>,
) {
    for sim in sim.iter() {
        // round pos down
        let pos = Vec2::new(pos.x.floor(), pos.y.floor());
        dbg_info.position = pos;
        dbg_info.chunk_position = Vec2::new((pos.x / CHUNK_SIZE.0 as f32).floor(), (pos.y / CHUNK_SIZE.1 as f32).floor());
        dbg_info.cell_position_in_chunk = Vec2::new((pos.x % CHUNK_SIZE.0 as f32).floor(), (pos.y % CHUNK_SIZE.1 as f32).floor());
        dbg_info.hovered_cell = sim.world.get_cell(pos.x as i32, pos.y as i32).copied();
    }
}

pub fn egui_ui(
    mut ctx: EguiContexts,
    mut dbg_info: ResMut<DebugInfo>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>
) {

    egui::Window::new("Debug Info")
    .show(ctx.ctx_mut(),
        |ui| {
            ui.set_min_width(200.0);

            ui.label(format!("Current State: {:?}", app_state.get()));
            ui.checkbox(&mut dbg_info.is_paused, "Paused");
            match dbg_info.is_paused {
                true => {
                    next_app_state.set(AppState::Paused);
                },
                false => {
                    next_app_state.set(AppState::Running);
                }
            }

            // convert to ms
            let sim_t_ms = dbg_info.average_frame_time() * 1000.0;
            let render_construct_t_ms = dbg_info.average_render_construct_time() * 1000.0;
            ui.label(format!("Sim Time: {:.2}ms", sim_t_ms));
            ui.label(format!("Render Construct Time: {:.2}ms", render_construct_t_ms));
            ui.label(format!("FPS: {:.2}", 1.0 / dbg_info.average_frame_time()));
            ui.label(format!("Position: {:?}", dbg_info.position));
            ui.label(format!("Hovered Cell: {:?}", dbg_info.hovered_cell));

            ui.separator();
            ui.heading("Chunk Information");
            ui.label(format!("Chunk Position: {:?}", dbg_info.chunk_position));
            ui.label(format!("Cell Position in Chunk: {:?}", dbg_info.cell_position_in_chunk));
            ui.checkbox(&mut dbg_info.show_gizmos, "Show Active Chunks");
        }
    );
}

pub fn keyboard_debug(
    keys: Res<bevy::input::ButtonInput<KeyCode>>,
    mut dbg_info: ResMut<DebugInfo>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        dbg_info.is_paused ^= true;
    }
}

pub fn cell_selector_ui(
    mut ctx: EguiContexts,
    mut pixel_interaction: ResMut<PixelSimulationInteraction>,
) {
    egui::Window::new("Cell Selector")
    .show(ctx.ctx_mut(),
        |ui| {
            ui.set_min_width(100.0);
            for (cell_type, name) in CellType::iter().zip(CellType::VARIANTS.iter()) {
                ui.radio_value(&mut pixel_interaction.selected_cell, cell_type, *name);
            }

            ui.add(egui::Slider::new(&mut pixel_interaction.cell_amount, 4..=100).text("Amount to spawn"));
        }
    );
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ChunkGizmos {}

pub fn draw_chunk_gizmos(
    mut chunk_gizmos: Gizmos<ChunkGizmos>,
    pixel_query: Query<&PixelSimulation>,
) {
    let sim = pixel_query.single();

    let awake_chunks = sim.world.get_awake_chunk_locs();

    // Draw a green rectangle for each awake chunk
    for (x, y) in awake_chunks {
        let pos_x = ((x as f32 * CHUNK_SIZE.0 as f32) - WORLD_SIZE.0 as f32 / 2.0) + CHUNK_SIZE.0 as f32 / 2.0;
        let pos_y = ((y as f32 * CHUNK_SIZE.1 as f32) - WORLD_SIZE.1 as f32 / 2.0) + CHUNK_SIZE.1 as f32 / 2.0;
        chunk_gizmos.rect_2d(
            Vec2::new(pos_x, pos_y),
            0.0,
            Vec2::new(CHUNK_SIZE.0 as f32, CHUNK_SIZE.1 as f32),
            Color::rgba(0.0, 1.0, 0.0, 0.5),
        );
    }
}

pub fn update_gizmos_config(
    mut config_store: ResMut<GizmoConfigStore>,
    mut dbg_info: ResMut<DebugInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let (chunk_config, _) = config_store.config_mut::<ChunkGizmos>();
    if dbg_info.show_gizmos {
        chunk_config.enabled = true;
    } else {
        chunk_config.enabled = false;
    }
    if keyboard.just_pressed(KeyCode::Digit0) {
        chunk_config.enabled ^= true;
        dbg_info.show_gizmos = chunk_config.enabled;
    }
}