use bevy::{
    math::{IVec2, UVec2},
    utils::{hashbrown::HashMap, syncunsafecell::SyncUnsafeCell},
};
use rand::Rng;

use super::{
    cell::{Cell, PhysicsType},
    chunk::PixelChunk,
    geometry_helpers::{
        BoundRect, DIRECTIONS, VEC_DOWN, VEC_DOWN_LEFT, VEC_DOWN_RIGHT, VEC_LEFT, VEC_RIGHT, VEC_UP,
    },
};

// SimulationChunkContext manages a 3x3 group of chunks temporarily while the updates happen
// It contains functions to help translate positions while updating and dealing with updating neighboring chunk data
pub struct SimulationChunkContext<'a> {
    // Position of the center chunk in the world's chunk map
    pub center_position: IVec2,
    // The nine chunks, including the center chunk (in the fourth position), Chunks are None if there is no neighbor there such as on world boundaries
    pub data: Vec<Option<&'a SyncUnsafeCell<PixelChunk>>>,

    // List of updated positions for each chunk
    pub dirty_updates: HashMap<IVec2, Vec<IVec2>>,

    chunk_size: UVec2,
}

impl SimulationChunkContext<'_> {
    // Create the contex using an vector of 3x3 chunks, asserted to be length 9
    pub fn new<'a>(
        center_position: IVec2,
        data: Vec<Option<&'a SyncUnsafeCell<PixelChunk>>>,
        chunk_size: UVec2,
    ) -> SimulationChunkContext<'a> {
        assert!(data.len() == 9);
        let mut dirty_updates = HashMap::new();
        for direction in DIRECTIONS {
            dirty_updates.insert(center_position + direction, Vec::new());
        }
        SimulationChunkContext {
            center_position,
            data,
            dirty_updates,
            chunk_size,
        }
    }

    fn get_rect(&self, chunk: usize) -> BoundRect {
        match self.data[chunk] {
            Some(c) => {
                let pc = unsafe { &*c.get() };
                pc.current_dirty_rect.union(&pc.previous_dirty_rect)
            }
            None => BoundRect::empty(),
        }
    }

    fn cell_from_index(&self, (chunk, index): (usize, usize)) -> &Cell {
        let chunk = unsafe { &*self.data[chunk].unwrap().get() };
        &chunk.cells[index]
    }

    // Transforms a 2d position into the 1d index
    fn cell_index(chunk_width: usize, pos: IVec2) -> usize {
        pos.x as usize + pos.y as usize * chunk_width
    }

    fn local_to_indices(&self, pos: IVec2) -> (usize, usize) {
        // Get the relative chunk position in the 3x3 group
        let rel_chunk = pos.div_euclid(self.chunk_size.as_ivec2());
        // Get cell position inside the chunk and convert into 1d position
        let cell_pos = Self::cell_index(
            self.chunk_size.x as usize,
            pos.rem_euclid(self.chunk_size.as_ivec2()),
        );

        (
            ((rel_chunk.x + 1) + (rel_chunk.y + 1) * 3) as usize,
            cell_pos,
        )
    }

    fn get_cell(&self, pos: IVec2) -> &Cell {
        self.cell_from_index(self.local_to_indices(pos))
    }

    fn cell_is_empty(&self, pos: IVec2) -> bool {
        let rel_chunk = pos.div_euclid(self.chunk_size.as_ivec2());
        let chunk = (rel_chunk.x + 1) as usize + (rel_chunk.y + 1) as usize * 3;
        match self.data[chunk] {
            Some(ch) => {
                let cell_pos = pos.rem_euclid(self.chunk_size.as_ivec2()).as_uvec2();
                let ch = unsafe { &*ch.get() };
                let cell = ch.cells[(cell_pos.x + cell_pos.y * self.chunk_size.x) as usize];
                cell.is_empty() && cell.updated == false
            }
            None => false,
        }
    }

    fn set_cell_from_index(&self, (chunk, index): (usize, usize), cell: Cell) {
        unsafe {
            (*self.data[chunk].unwrap().get()).cells[index] = cell;
        }
    }

    fn set_updated_cell_from_index(&self, (chunk, index): (usize, usize)) {
        unsafe { (*self.data[chunk].unwrap().get()).cells[index].updated = true }
    }

    fn set_cell(&mut self, pos: IVec2, cell: Cell) {
        let idx = self.local_to_indices(pos);
        self.set_cell_from_index(idx, cell);
        self.update_dirty_idx(pos);
        // If the cell is on the side, update the adjacent chunks dirty rect
        if pos.x == 0 {
            self.update_dirty_idx(pos + IVec2::X * -1);
        } else if pos.x == self.chunk_size.x as i32 - 1 {
            self.update_dirty_idx(pos + IVec2::X);
        }
        if pos.y == 0 {
            self.update_dirty_idx(pos + IVec2::Y * -1);
        } else if pos.y == self.chunk_size.y as i32 - 1 {
            self.update_dirty_idx(pos + IVec2::Y);
        }
    }

    fn set_updated_cell(&mut self, pos: IVec2) {
        let idx = self.local_to_indices(pos);
        self.set_updated_cell_from_index(idx);
        self.update_dirty_idx(pos);
        // If the cell is on the side, update the adjacent chunks dirty rect
        if pos.x == 0 {
            self.update_dirty_idx(pos + IVec2::X * -1);
        } else if pos.x == self.chunk_size.x as i32 - 1 {
            self.update_dirty_idx(pos + IVec2::X);
        }
        if pos.y == 0 {
            self.update_dirty_idx(pos + IVec2::Y * -1);
        } else if pos.y == self.chunk_size.y as i32 - 1 {
            self.update_dirty_idx(pos + IVec2::Y);
        }
    }

    fn update_dirty_idx(&mut self, pos: IVec2) {
        let rel_chunk = pos.div_euclid(self.chunk_size.as_ivec2());
        let chunk_vec = self
            .dirty_updates
            .get_mut(&(self.center_position + rel_chunk));
        if let Some(vec) = chunk_vec {
            vec.push(pos.rem_euclid(self.chunk_size.as_ivec2()))
        }
    }

    // Simulates the chunks based on the center chunk's dirty rect
    pub fn simulate(&mut self) -> HashMap<IVec2, Vec<IVec2>> {
        const CENTER: usize = 4;

        let center_rect = self.get_rect(CENTER).clone();

        // Iterate over dirty rect
        for y in center_rect.min.y..=center_rect.max.y {
            // Alternate x direction
            if rand::thread_rng().gen_bool(0.5) {
                for x in center_rect.min.x..=center_rect.max.x {
                    // Process this cell and set it with the result of the process
                    if let Some(cell) = self.process_cell(IVec2 { x, y }) {
                        self.set_cell(
                            IVec2 { x, y },
                            Cell {
                                updated: false,
                                ..cell
                            },
                        )
                    }
                }
            } else {
                for x in (center_rect.min.x..=center_rect.max.x).rev() {
                    // Process this cell and set it with the result of the process
                    if let Some(cell) = self.process_cell(IVec2 { x, y }) {
                        self.set_cell(
                            IVec2 { x, y },
                            Cell {
                                updated: false,
                                ..cell
                            },
                        )
                    }
                }
            }
        }

        self.dirty_updates.clone()
    }

    fn move_down(&mut self, current: Cell, position: IVec2) -> Option<Cell> {
        if rand::thread_rng().gen_bool(0.5)
            && self.cell_is_empty(IVec2 {
                x: position.x,
                y: position.y - 2,
            })
        {
            self.set_cell(
                IVec2 {
                    x: position.x,
                    y: position.y - 2,
                },
                current,
            );
            // Set intermediate as updated
            self.set_updated_cell(IVec2 {
                x: position.x,
                y: position.y - 1,
            });
        } else {
            self.set_cell(
                IVec2 {
                    x: position.x,
                    y: position.y - 1,
                },
                current,
            );
        }
        Some(Cell::default())
    }

    fn move_up(&mut self, current: Cell, position: IVec2) -> Option<Cell> {
        if rand::thread_rng().gen_bool(0.5)
            && self.cell_is_empty(IVec2 {
                x: position.x,
                y: position.y + 2,
            })
        {
            self.set_cell(
                IVec2 {
                    x: position.x,
                    y: position.y + 2,
                },
                current,
            );
            // Set intermediate as updated
            self.set_updated_cell(IVec2 {
                x: position.x,
                y: position.y + 1,
            });
        } else {
            self.set_cell(
                IVec2 {
                    x: position.x,
                    y: position.y + 1,
                },
                current,
            );
        }
        Some(Cell::default())
    }

    fn move_left_right(
        &mut self,
        current: Cell,
        position: IVec2,
        move_left: bool,
        move_right: bool,
    ) -> Option<Cell> {
        if move_left && move_right {
            // choose random direction
            self.set_cell(
                IVec2 {
                    x: if rand::thread_rng().gen_bool(0.5) {
                        position.x + 1
                    } else {
                        position.x - 1
                    },
                    ..position
                },
                current,
            );
            return Some(Cell::default());
        } else if move_left {
            // move 2 if possible
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(IVec2 {
                    x: position.x - 2,
                    ..position
                })
            {
                self.set_cell(
                    IVec2 {
                        x: position.x - 2,
                        ..position
                    },
                    current,
                );
                // Set intermediate cell updated
                self.set_updated_cell(IVec2 {
                    x: position.x - 1,
                    ..position
                });
            } else {
                self.set_cell(
                    IVec2 {
                        x: position.x - 1,
                        ..position
                    },
                    current,
                );
            }
            return Some(Cell::default());
        } else if move_right {
            // move 2 if possible
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(IVec2 {
                    x: position.x + 2,
                    ..position
                })
            {
                self.set_cell(
                    IVec2 {
                        x: position.x + 2,
                        ..position
                    },
                    current,
                );
                // Set intermediate cell updated
                self.set_updated_cell(IVec2 {
                    x: position.x + 1,
                    ..position
                });
            } else {
                self.set_cell(
                    IVec2 {
                        x: position.x + 1,
                        ..position
                    },
                    current,
                );
            }
            return Some(Cell::default());
        }
        None
    }

    fn move_down_left_right(
        &mut self,
        current: Cell,
        position: IVec2,
        move_left: bool,
        move_right: bool,
    ) -> Option<Cell> {
        if move_left && move_right {
            // choose random direction
            self.set_cell(
                IVec2 {
                    x: if rand::thread_rng().gen_bool(0.5) {
                        position.x + 1
                    } else {
                        position.x - 1
                    },
                    y: position.y - 1,
                },
                current,
            );
            return Some(Cell::default());
        } else if move_left {
            // move 2
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(IVec2 {
                    x: position.x - 2,
                    y: position.y - 2,
                })
            {
                self.set_cell(
                    IVec2 {
                        x: position.x - 2,
                        y: position.y - 2,
                    },
                    current,
                );
                // Set intermediate cell updated
                self.set_updated_cell(IVec2 {
                    x: position.x - 1,
                    y: position.y - 1,
                });
            } else {
                self.set_cell(
                    IVec2 {
                        x: position.x - 1,
                        y: position.y - 1,
                    },
                    current,
                );
            }
            return Some(Cell::default());
        } else if move_right {
            // move 2
            if rand::thread_rng().gen_bool(0.5)
                && self.cell_is_empty(IVec2 {
                    x: position.x + 2,
                    y: position.y - 2,
                })
            {
                self.set_cell(
                    IVec2 {
                        x: position.x + 2,
                        y: position.y - 2,
                    },
                    current,
                );
                // Set intermediate cell updated
                self.set_updated_cell(IVec2 {
                    x: position.x + 1,
                    y: position.y - 1,
                });
            } else {
                self.set_cell(
                    IVec2 {
                        x: position.x + 1,
                        y: position.y - 1,
                    },
                    current,
                );
            }
            return Some(Cell::default());
        }
        None
    }

    // Simulates a single cell, given by it's position in the chunk
    // Uses the chunk context to manipulate the surroundings
    fn process_cell(&mut self, position: IVec2) -> Option<Cell> {
        let mut current = self.get_cell(position).clone();

        if current.updated {
            return None;
        } else {
            current.updated = true;
        }
        let mut new = None;

        match current.physics {
            PhysicsType::Empty => {}
            PhysicsType::SoftSolid(_) => {
                let down_empty = self.cell_is_empty(position + VEC_DOWN);
                let down_left_empty = self.cell_is_empty(position + VEC_DOWN_LEFT);
                let down_right_empty = self.cell_is_empty(position + VEC_DOWN_RIGHT);

                if down_empty
                    && (!(down_left_empty || down_right_empty)
                        || rand::thread_rng().gen_range(0..10) != 0)
                {
                    new = self.move_down(current, position);
                } else {
                    new = self.move_down_left_right(
                        current,
                        position,
                        down_left_empty,
                        down_right_empty,
                    );
                }
            }
            PhysicsType::Liquid(_) => {
                let down_empty = self.cell_is_empty(position + VEC_DOWN);
                let left_empty = self.cell_is_empty(position + VEC_LEFT);
                let right_empty = self.cell_is_empty(position + VEC_RIGHT);

                if down_empty && (!(left_empty || right_empty) || rand::thread_rng().gen_bool(0.95))
                {
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
            _ => {}
        }
        new
    }
}
