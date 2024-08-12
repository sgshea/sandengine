//! Interaction with pixel world

use bevy::prelude::*;

use bevy::math::IVec2;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};
use strum::{IntoEnumIterator, VariantNames};

use super::cell::{Cell, CellType};
use super::PixelSimulation;

#[derive(Resource)]
pub struct PixelInteraction {
    // Hovered position in world coordinates
    pub hovered_position: IVec2,
    // Type of cell to be placed on click
    pub place_cell_type: CellType,
    // Amount of cell to place
    pub place_cell_amount: i32,
}

impl Default for PixelInteraction {
    fn default() -> Self {
        Self {
            hovered_position: IVec2::ZERO,
            place_cell_amount: 8,
            place_cell_type: CellType::Sand,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
        app.add_systems(Update, get_position);

        app.init_resource::<PixelInteraction>();
        app.add_systems(Update, pixel_interaction_config);
        app.add_systems(FixedPostUpdate, handle_mouse_input);
}

fn get_position(
    mut pxl: ResMut<PixelInteraction>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let cursor_screen_position = primary_window.single().cursor_position();

    if cursor_screen_position.is_none() {
        return
    }
    let (cam, trans) = camera.single();

    let world_pos = cam.viewport_to_world_2d(trans, cursor_screen_position.unwrap()).unwrap();

    let cell_pos = world_pos.as_ivec2();
    pxl.hovered_position = cell_pos;
}

fn pixel_interaction_config(
    mut ctx: EguiContexts,
    mut pxl: ResMut<PixelInteraction>,
) {
    egui::Window::new("Pixel Simulation").show(ctx.ctx_mut(),
        | ui | {
            ui.set_min_width(200.);
            for (cell_type, name) in CellType::iter().zip(CellType::VARIANTS.iter()) {
                ui.radio_value(&mut pxl.place_cell_type, cell_type, *name);
            }

            ui.add(egui::Slider::new(&mut pxl.place_cell_amount, 8..=100).text("Amount to spawn"));
        }
    );
}

// Intended to be called with cell type
fn place_cells(
    mut sim: Query<&mut PixelSimulation>,
    position: IVec2,
    amount: i32,
    cell_type: CellType,
) {
    let amt_to_place_quarter = amount / 4;
    let amt_to_place_half = amount / 2;
    let world = &mut sim.single_mut().world;
    for x in -amt_to_place_half..=amt_to_place_half {
        for y in -amt_to_place_half..amt_to_place_half {
            // Make circle
            if (x * x) + (y * y) > amt_to_place_quarter * amt_to_place_quarter {
                continue;
            }
            world.set_cell(position + IVec2 { x, y }, Cell::from(cell_type));
        }
    }
}

fn handle_mouse_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    sim: Query<&mut PixelSimulation>,
    pxl: ResMut<PixelInteraction>,
) {
    if mouse_buttons.pressed(MouseButton::Left) {
        place_cells(sim, pxl.hovered_position, pxl.place_cell_amount, pxl.place_cell_type);
    }
    else if mouse_buttons.pressed(MouseButton::Right) { 
        place_cells(sim, pxl.hovered_position, pxl.place_cell_amount, CellType::Empty);
    }
}