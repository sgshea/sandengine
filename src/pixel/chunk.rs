use bevy::math::IVec2;

use crate::CHUNK_SIZE;

use super::{cell::{Cell, PhysicsType}, geometry_helpers::BoundRect};

#[derive(Debug, Clone)]
pub struct PixelChunk {
    pub current_dirty_rect: BoundRect,
    pub previous_dirty_rect: BoundRect,

    // Chunk position
    pub position: IVec2,

    pub cells: Vec<Cell>,
}

impl PixelChunk {
    pub fn new(width: i32, height: i32, pos_x: i32, pos_y: i32) -> Self {
        let cells = vec![Cell::default(); (height * width) as usize];

        PixelChunk {
            position: IVec2 { x: pos_x, y: pos_y },
            current_dirty_rect: BoundRect {
                    min: IVec2::ZERO,
                    // We iterate over the range of the BoundRect to the end (inclusive) so we need to subtract 1 to not go out of bounds
                    max: CHUNK_SIZE - 1,
                    },
            previous_dirty_rect: BoundRect::empty(),
            cells,
        }
    }

    pub fn should_update(&self) -> bool {
        !self.current_dirty_rect.is_empty()
    }

    pub fn get_index(&self, x: i32, y: i32) -> usize {
        (y * CHUNK_SIZE.x + x) as usize
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
        self.current_dirty_rect = self.current_dirty_rect.union_point(&IVec2::new(x, y));
    }

    pub fn construct_dirty_rect(&mut self, points: &[IVec2]) {
        let new_rect = BoundRect::from_points(points);
        self.current_dirty_rect = new_rect;
    }

    pub fn swap_rects(&mut self) {
        self.previous_dirty_rect = self.current_dirty_rect.union(&self.previous_dirty_rect);
        self.current_dirty_rect = BoundRect::empty();
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