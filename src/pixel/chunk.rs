use bevy::math::{IRect, IVec2};

use crate::CHUNK_SIZE;

use super::cell::{Cell, PhysicsType};

#[derive(Debug, Clone)]
pub struct PixelChunk {
    // Will become dirty rect in future version
    pub boundary_rect: IRect,

    // Chunk position
    pub position: IVec2,

    pub cells: Vec<Cell>,
}

impl PixelChunk {
    pub fn new(width: i32, height: i32, pos_x: i32, pos_y: i32) -> Self {
        let cells = vec![Cell::default(); (height * width) as usize];

        PixelChunk {
            position: IVec2 { x: pos_x, y: pos_y },
            boundary_rect: IRect {
                    min: IVec2::ZERO,
                    max: CHUNK_SIZE,
                },
            cells,
        }
    }

    pub fn get_index(&self, x: i32, y: i32) -> usize {
        (y * self.boundary_rect.width() + x) as usize
    }

    pub fn get_cell(&self, position: IVec2) -> Cell {
        let idx = self.get_index(position.x, position.y);
        self.cells[idx]
    }

    pub fn set_cell_1d(&mut self, idx: usize, cell: Cell) {
        if idx < self.cells.len() {
            self.cells[idx] = cell;
        }
    }

    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        let idx = self.get_index(x, y);
        self.set_cell_1d(idx, cell);
    }

    // Reset all cells to not be updated
    pub fn commit_cells_unupdated(&mut self) {
        self.cells.iter_mut().for_each(|cell| {
            cell.updated = false;
        });
    }

    pub fn cells_as_floats(&self) -> Vec<f64> {
        // Map each cell to a float depending on if it is solid
        // range 0.0-1.0

        self.cells.iter().map(|cell| {
            if cell.physics == PhysicsType::Empty {
                0.0
            } else {
                1.0
            }
        }).collect::<Vec<f64>>()
    }
}