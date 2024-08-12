//! SimulationChunkContext manages a 3x3 group of chunks temporarily while the updates happen
//! It contains functions to help translate positions while updating and dealing with updating neighboring chunk data
use std::cell::UnsafeCell;

use bevy::math::{IRect, IVec2};
use rand::Rng;

use crate::CHUNK_SIZE;

use super::{cell::{Cell, PhysicsType}, chunk::PixelChunk, geometry_helpers::{VEC_DOWN, VEC_DOWN_LEFT, VEC_DOWN_RIGHT, VEC_LEFT, VEC_RIGHT, VEC_UP}};

pub struct SimulationChunkContext<'a, 'b> {
    // Position of the center chunk in the world's chunk map
    pub center_position: IVec2,
    // The nine chunks, including the center chunk (in the fourth position), Chunks are None if there is no neighbor there such as on world boundaries
    pub data: &'a mut [Option<&'b UnsafeCell<PixelChunk>>; 9],
}

impl SimulationChunkContext<'_, '_> {
    fn get_rect(&self, chunk: usize) -> Option<IRect> {
        match self.data[chunk] {
            Some(c) => {
                let pc = unsafe { &*c.get() };
                Some(pc.boundary_rect)
            },
            None => None,
        }
    }

    fn cell_from_index(
        &self,
        (chunk, index): (usize, usize)
    ) -> &Cell {
        let chunk = unsafe { &*self.data[chunk].unwrap().get() };
        &chunk.cells[index]
    }

    fn cell_index(chunk_width: usize, pos: IVec2) -> usize {
        pos.x as usize + pos.y as usize * chunk_width
    }

    fn local_to_indices(
        pos: IVec2
    ) -> (usize, usize) {
        // Get the relative chunk position in the 3x3 group
        let rel_chunk = pos.div_euclid(CHUNK_SIZE);
        // Get cell position inside the chunk and convert into 1d position
        let cell_pos = Self::cell_index(CHUNK_SIZE.x as usize, pos.rem_euclid(CHUNK_SIZE));

        (
            ((rel_chunk.x + 1) + (rel_chunk.y + 1) * 3) as usize,
            cell_pos
        )
    }

    fn get_cell(&self, pos: IVec2) -> &Cell {
        self.cell_from_index(Self::local_to_indices(pos))
    }

    fn cell_is_empty(&self, pos: IVec2) -> bool {
        let rel_chunk = pos.div_euclid(CHUNK_SIZE);
        let chunk = (rel_chunk.x + 1) as usize + (rel_chunk.y + 1) as usize * 3;
        match self.data[chunk] {
            Some(ch) => {
                let cell_pos = pos.rem_euclid(CHUNK_SIZE).as_uvec2();
                let ch = unsafe { &*ch.get() };
                let cell = ch.cells[(cell_pos.x + cell_pos.y * CHUNK_SIZE.x as u32) as usize];
                cell.is_empty() && cell.updated == false
            },
            None => false,
        }
    }

    fn set_cell_from_index (
        &self,
        (chunk, index): (usize, usize),
        cell: Cell
    ) {
        unsafe {
            (*self.data[chunk].unwrap().get()).cells[index] = cell;
        }
    }

    fn set_updated_cell_from_index(
        &self,
        (chunk, index): (usize, usize),
    ) {
        unsafe {
            (*self.data[chunk].unwrap().get()).cells[index].updated = true
        }
    }

    fn set_cell(&mut self, pos: IVec2, cell: Cell)  {
        let idx = Self::local_to_indices(pos);
        self.set_cell_from_index(idx, cell);
    }

    fn set_updated_cell(&mut self, pos: IVec2)  {
        let idx = Self::local_to_indices(pos);
        self.set_updated_cell_from_index(idx);
    }

    pub fn simulate(&mut self) {
        const CENTER: usize = 4;

        let center_rect = self.get_rect(CENTER).expect("Center chunk should always exist");

        // Iterate over dirty rect
        for y in center_rect.min.y..center_rect.max.y {
            // Alternate x direction
            if rand::thread_rng().gen_bool(0.5) {
                for x in center_rect.min.x..center_rect.max.x {
                    // Process this cell and set it with the result of the process
                    if let Some(cell) = self.process_cell(IVec2 { x, y }) {
                        self.set_cell(IVec2 { x, y }, Cell {
                            updated: false,
                            ..cell
                        })
                    }
                }
            } else {
                for x in (center_rect.min.x..center_rect.max.x).rev() {
                    // Process this cell and set it with the result of the process
                    if let Some(cell) = self.process_cell(IVec2 { x, y }) {
                        self.set_cell(IVec2 { x, y }, Cell {
                            updated: false,
                            ..cell
                        })
                    }
                }
            }
        }
    }

    fn move_down(&mut self, current: Cell, position: IVec2) -> Option<Cell> {
        if rand::thread_rng().gen_bool(0.5) && self.cell_is_empty(IVec2 { x: position.x, y: position.y - 2 }) {
            self.set_cell(IVec2 { x: position.x, y: position.y - 2 }, current);
            // Set intermediate as updated
            self.set_updated_cell(IVec2 { x: position.x, y: position.y - 1 });
        } else {
            self.set_cell(IVec2 { x: position.x, y: position.y - 1 }, current);
        }
        Some(Cell::default())
    }

    fn move_up(&mut self, current: Cell, position: IVec2) -> Option<Cell> {
        if rand::thread_rng().gen_bool(0.5) && self.cell_is_empty(IVec2 { x: position.x, y: position.y + 2 }) {
            self.set_cell(IVec2 { x: position.x, y: position.y + 2 }, current);
            // Set intermediate as updated
            self.set_updated_cell(IVec2 { x: position.x, y: position.y + 1 });
        } else {
            self.set_cell(IVec2 { x: position.x, y: position.y + 1 }, current);
        }
        Some(Cell::default())
    }

    fn move_left_right(&mut self, current: Cell, position: IVec2, move_left: bool, move_right: bool) -> Option<Cell> {
        if move_left && move_right {
            // choose random direction
            self.set_cell(
                IVec2 { 
                    x: if rand::thread_rng().gen_bool(0.5) { position.x + 1 } else { position.x - 1 },
                    ..position
                },
                current,
            );
            return Some(Cell::default())
        } else if move_left {
            // move 2 if possible
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(
                    IVec2 {
                        x: position.x - 2,
                        ..position
                        },
                    )
            {
                self.set_cell(
                    IVec2 { x: position.x - 2, ..position },
                    current
                );
                // Set intermediate cell updated
                self.set_updated_cell(
                    IVec2 { x: position.x - 1, ..position },
                );
            } else {
                self.set_cell(
                    IVec2 { x: position.x - 1, ..position },
                    current
                );
            }
            return Some(Cell::default())
        } else if move_right {
            // move 2 if possible
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(
                    IVec2 {
                        x: position.x + 2,
                        ..position
                        },
                    )
            {
                self.set_cell(
                    IVec2 { x: position.x + 2, ..position },
                    current
                );
                // Set intermediate cell updated
                self.set_updated_cell(
                    IVec2 { x: position.x + 1, ..position },
                );
            } else {
                self.set_cell(
                    IVec2 { x: position.x + 1, ..position },
                    current
                );
            }
            return Some(Cell::default())
        }
        None
    }

    fn move_down_left_right(&mut self, current: Cell, position: IVec2, move_left: bool, move_right: bool) -> Option<Cell> {
        if move_left && move_right {
            // choose random direction
            self.set_cell(
                IVec2 { 
                    x: if rand::thread_rng().gen_bool(0.5) { position.x + 1 } else { position.x - 1 },
                    y: position.y - 1
                },
                current,
            );
            return Some(Cell::default())
        } else if move_left {
            // move 2
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(
                    IVec2 {
                        x: position.x - 2,
                        y: position.y - 2
                        },
                    )
            {
                self.set_cell(
                    IVec2 { x: position.x - 2, y: position.y - 2 },
                    current
                );
                // Set intermediate cell updated
                self.set_updated_cell(
                    IVec2 { x: position.x - 1, y: position.y - 1 },
                );
            } else {
                self.set_cell(
                    IVec2 { x: position.x - 1, y: position.y - 1 },
                    current
                );
            }
            return Some(Cell::default())
        } else if move_right {
            // move 2
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(
                    IVec2 {
                        x: position.x + 2,
                        y: position.y - 2
                        },
                    )
            {
                self.set_cell(
                    IVec2 { x: position.x + 2, y: position.y - 2 },
                    current
                );
                // Set intermediate cell updated
                self.set_updated_cell(
                    IVec2 { x: position.x + 1, y: position.y - 1 },
                );
            } else {
                self.set_cell(
                    IVec2 { x: position.x + 1, y: position.y - 1 },
                    current
                );
            }
            return Some(Cell::default())
        }
        None
    }

    fn move_up_left_right(&mut self, current: Cell, position: IVec2, move_left: bool, move_right: bool) -> Option<Cell> {
        if move_left && move_right {
            // choose random direction
            self.set_cell(
                IVec2 { 
                    x: if rand::thread_rng().gen_bool(0.5) { position.x + 1 } else { position.x - 1 },
                    y: position.y + 1
                },
                current,
            );
            return Some(Cell::default())
        } else if move_left {
            // move 2
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(
                    IVec2 {
                        x: position.x - 2,
                        y: position.y + 2
                        },
                    )
            {
                self.set_cell(
                    IVec2 { x: position.x - 2, y: position.y + 2 },
                    current
                );
                // Set intermediate cell updated
                self.set_updated_cell(
                    IVec2 { x: position.x - 1, y: position.y + 1 },
                );
            } else {
                self.set_cell(
                    IVec2 { x: position.x - 1, y: position.y + 1 },
                    current
                );
            }
            return Some(Cell::default())
        } else if move_right {
            // move 2
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(
                    IVec2 {
                        x: position.x + 2,
                        y: position.y + 2
                        },
                    )
            {
                self.set_cell(
                    IVec2 { x: position.x + 2, y: position.y + 2 },
                    current
                );
                // Set intermediate cell updated
                self.set_updated_cell(
                    IVec2 { x: position.x + 1, y: position.y + 1 },
                );
            } else {
                self.set_cell(
                    IVec2 { x: position.x + 1, y: position.y + 1 },
                    current
                );
            }
            return Some(Cell::default())
        }
        None
    }

    // Simulates a single cell, given by it's position in the chunk
    // Uses the chunk context to manipulate the surroundings
    fn process_cell(
        &mut self,
        position: IVec2,
    ) -> Option<Cell> {
        let mut current = self.get_cell(position).clone();
        if current.updated {
            return None
        } else {
            current.updated = true;
        }
        let mut new = None;

        match current.physics {
            PhysicsType::Empty => {},
            PhysicsType::SoftSolid(_) => {
                let down_empty = self.cell_is_empty(position + VEC_DOWN);
                let down_left_empty = self.cell_is_empty(position + VEC_DOWN_LEFT);
                let down_right_empty = self.cell_is_empty(position + VEC_DOWN_RIGHT);

                if down_empty && (!(down_left_empty || down_right_empty) || rand::thread_rng().gen_range(0..10) != 0) {
                    new = self.move_down(current, position);
                } else {
                    new = self.move_down_left_right(current, position, down_left_empty, down_right_empty);
                }
            }
            PhysicsType::Liquid(_) => {
                let down_empty = self.cell_is_empty(position + VEC_DOWN);
                let left_empty = self.cell_is_empty(position + VEC_LEFT);
                let right_empty = self.cell_is_empty(position + VEC_RIGHT);

                if down_empty && (!(left_empty || right_empty) || rand::thread_rng().gen_bool(0.95)) {
                    new = self.move_down(current, position);
                } else {
                    new = self.move_left_right(current, position, left_empty, right_empty);
                }
            }
            PhysicsType::Gas(_) => {
                let up_empty = self.cell_is_empty(position + VEC_UP);
                let left_empty = self.cell_is_empty(position + VEC_LEFT);
                let right_empty = self.cell_is_empty(position + VEC_RIGHT);

                if up_empty && (!(left_empty || right_empty) || rand::thread_rng().gen_bool(0.95)) {
                    new = self.move_up(current, position);
                } else {
                    new = self.move_left_right(current, position, left_empty, right_empty);
                }
            }
            _ => {},
        }
        new
    }
}

