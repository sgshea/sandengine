//! Debug egui window for information directly relating to pixel world

use bevy::color::palettes::css::{LIGHT_GRAY, LIGHT_GREEN};
use bevy::prelude::*;

use bevy::math::IVec2;
use bevy_egui::{egui, EguiContexts};

use crate::dev_tools::PixelSimulationDebugUi;
use crate::input::InteractionInformation;
use crate::states::{AppSet, DebugState};

use super::cell::Cell;
use super::world::PixelWorld;

// Debug information to be stored for the pixel world
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
    app.init_resource::<PixelSimulationDebug>();
    app.add_systems(
        Update,
        (pixel_simulation_debug, pixel_simulation_debug_ui)
            .run_if(in_state(DebugState::ShowAll))
            .in_set(AppSet::Update),
    );
    app.init_gizmo_group::<ChunkGizmos>();
    app.add_systems(
        PostUpdate,
        (draw_chunk_gizmos, update_pixel_debug_gizmos)
            .run_if(in_state(DebugState::ShowAll))
            .in_set(AppSet::Update),
    );
}

fn pixel_simulation_debug(
    sim: Query<&mut PixelWorld>,
    mut dbg: ResMut<PixelSimulationDebug>,
    int: Res<InteractionInformation>,
) {
    let world = match sim.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let cell_pos = int.mouse_position.as_ivec2();
    dbg.position_in_chunk = PixelWorld::cell_to_position_in_chunk(world.chunk_size, cell_pos);
    dbg.chunk_position = PixelWorld::cell_to_chunk_position(world.chunk_size, cell_pos);
    dbg.hovered_cell = world.get_cell(cell_pos);
    dbg.inside_dirty_rect = world.cell_inside_dirty(cell_pos);

    dbg.chunk_size = world.get_chunk_width() as u32;
    dbg.chunk_amount = world.get_chunks().len() as u32;
}

fn pixel_simulation_debug_ui(
    mut ctx: EguiContexts,
    mut dbg: ResMut<PixelSimulationDebug>,
    mut dbg_ui: ResMut<PixelSimulationDebugUi>,
    int: Res<InteractionInformation>,
) {
    egui::Window::new("Debug")
        .open(&mut dbg_ui.show)
        .show(ctx.ctx_mut(), |ui| {
            ui.set_min_width(200.);
            ui.label(format!("Current Chunk: {:?}", dbg.chunk_position));
            ui.label(format!("Current Cell: {:?}", dbg.hovered_cell));
            ui.label(format!("Inside dirty rect?: {:?}", dbg.inside_dirty_rect));
            ui.label(format!(
                "Cell position in world: {:?}",
                int.mouse_position.as_ivec2()
            ));
            ui.label(format!(
                "Cell position in chunk: {:?}",
                dbg.position_in_chunk
            ));
            ui.separator();
            ui.label(format!(
                "Amount of chunks/chunk size: {:?}/{:?}",
                dbg.chunk_amount, dbg.chunk_size
            ));
            ui.checkbox(&mut dbg.show_chunk_borders, "F2: Toggle chunk overlay, gray outline for chunks,\ngreen outline for dirty rectangles");
            ui.label("F3: Toggle Rapier Physics Engine Debug Overlay");
        });
}

// Gizmos for dirty rect and chunk borders
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ChunkGizmos {}

pub fn draw_chunk_gizmos(mut chunk_gizmos: Gizmos<ChunkGizmos>, sim: Query<&PixelWorld>) {
    let world = match sim.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let origin = world.world_size.as_vec2() / 2.;

    let awake_chunks = world.get_chunk_dirty_rects();

    for (pos, rect) in awake_chunks {
        // Calculate position in screen
        let pos = (pos.as_vec2() * world.chunk_size.as_vec2()) - world.world_size.as_vec2() / 2.;

        // Draw light gray outline of chunk
        chunk_gizmos.rect_2d(
            origin + pos + (world.chunk_size.as_vec2() / 2.),
            0.0,
            world.chunk_size.as_vec2(),
            LIGHT_GRAY,
        );
        // Draw green outline of dirty rect if exists
        if !rect.is_empty() {
            chunk_gizmos.rect_2d(
                origin + pos + rect.center_display(),
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
