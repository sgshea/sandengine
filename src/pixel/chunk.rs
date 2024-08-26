use bevy::math::{IVec2, UVec2};

use super::{cell::{Cell, PhysicsType}, geometry_helpers::BoundRect};

#[derive(Debug, Clone)]
pub struct PixelChunk {
    pub current_dirty_rect: BoundRect,
    pub previous_dirty_rect: BoundRect,
    // Force chunk to re-render while this value counts down to 0
    pub render_override: u8,

    // Chunk position
    pub position: IVec2,
    pub size: UVec2,

    pub cells: Vec<Cell>,
}

impl PixelChunk {
    pub fn new(size: UVec2, position: IVec2) -> Self {
        let cells = vec![Cell::default(); (size.x * size.y) as usize];

        PixelChunk {
            position,
            render_override: 0,
            current_dirty_rect: BoundRect {
                    min: IVec2::ZERO,
                    // We iterate over the range of the BoundRect to the end (inclusive) so we need to subtract 1 to not go out of bounds
                    max: (size - UVec2::ONE).as_ivec2(),
                    },
            previous_dirty_rect: BoundRect::empty(),
            size,
            cells,
        }
    }

    pub fn should_update(&self) -> bool {
        !self.current_dirty_rect.is_empty() || self.render_override > 0
    }

    pub fn get_index(&self, x: i32, y: i32) -> usize {
        (y * self.size.x as i32 + x) as usize
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
        if self.current_dirty_rect.is_empty() {
            self.current_dirty_rect = self.current_dirty_rect.union_point_plus(&IVec2::new(x, y));
        } else {
            self.current_dirty_rect = self.current_dirty_rect.union_point(&IVec2::new(x, y));
        }
    }

    pub fn construct_dirty_rect(&mut self, points: &[IVec2]) {
        let new_rect = BoundRect::from_points(points);
        self.current_dirty_rect = new_rect;
        // Reset override
        if self.current_dirty_rect.is_empty() && self.render_override > 0 {
            self.render_override -= 1;
        }
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

    // Convert the grid to a byte array for rendering
    pub fn render_chunk(&self) -> Vec<u8> {
        self.cells.iter().flat_map(|cell| {
            cell.color
        }).collect::<Vec<u8>>()
    }
}