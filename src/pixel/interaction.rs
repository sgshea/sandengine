//! Interaction with pixel world

use bevy::prelude::*;

use bevy::math::IVec2;
use bevy_egui::{egui, EguiContexts};
use strum::{IntoEnumIterator, VariantNames};

use crate::input::InteractionInformation;
use crate::screen::Screen;

use super::cell::{Cell, CellType};
use super::world::PixelWorld;

// Information about interacting with the pixel world
#[derive(Resource)]
pub struct PixelInteraction {
    // Type of cell to be placed on click
    pub place_cell_type: CellType,
    // Amount of cell to place
    pub place_cell_amount: i32,
}

impl Default for PixelInteraction {
    fn default() -> Self {
        Self {
            place_cell_amount: 8,
            place_cell_type: CellType::Sand,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PixelInteraction>();
    app.add_systems(
        Update,
        (pixel_interaction_config, handle_mouse_input).run_if(in_state(Screen::Playing)),
    );
}

fn pixel_interaction_config(mut ctx: EguiContexts, mut pxl: ResMut<PixelInteraction>) {
    egui::Window::new("Pixel Simulation").show(ctx.ctx_mut(),
        | ui | {
            ui.set_min_width(200.);
            ui.label("Controls:");
            ui.label("Left click to place the selected cell type below");
            ui.label("Hold left control and press left click to erase the selected cell type or select empty");
            ui.separator();
            for (cell_type, name) in CellType::iter().zip(CellType::VARIANTS.iter()) {
                ui.radio_value(&mut pxl.place_cell_type, cell_type, *name);
            }

            ui.add(egui::Slider::new(&mut pxl.place_cell_amount, 8..=100).text("Size of brush"));
        }
    );
}

// Intended to be called with cell type
fn place_cells(world: &mut PixelWorld, position: IVec2, amount: i32, cell_type: CellType) {
    let amt_to_place_quarter = amount / 4;
    let amt_to_place_half = amount / 2;
    for x in -amt_to_place_half..=amt_to_place_half {
        for y in -amt_to_place_half..amt_to_place_half {
            // Make circle
            if (x * x) + (y * y) > amt_to_place_quarter * amt_to_place_quarter {
                continue;
            }
            world.set_cell_external(position + IVec2 { x, y }, Cell::from(cell_type));
        }
    }
}

fn handle_mouse_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard_buttons: Res<ButtonInput<KeyCode>>,
    mut sim: Query<&mut PixelWorld>,
    pxl: ResMut<PixelInteraction>,
    int: Res<InteractionInformation>,
) {
    // Don't do anything if we are hovering over UI
    if int.hovering_ui {
        return;
    }

    let world = &mut sim.single_mut();

    if mouse_buttons.pressed(MouseButton::Left) {
        // Delete cells if control is held
        if keyboard_buttons.pressed(KeyCode::ControlLeft) {
            place_cells(
                world,
                int.mouse_position.as_ivec2(),
                pxl.place_cell_amount,
                CellType::Empty,
            );
        } else {
            place_cells(
                world,
                int.mouse_position.as_ivec2(),
                pxl.place_cell_amount,
                pxl.place_cell_type,
            );
        }
    }
}
