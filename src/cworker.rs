

use std::mem;

use bevy::utils::hashbrown::HashMap;

use crate::{cell::Cell, cell_types::{CellType, DirectionType, StateType}, chunk::{PixelChunk, SplitChunk}};

pub struct ChunkWorker<'a> {
    chunk: &'a mut PixelChunk,
    surrounding_chunks: HashMap<(i32, i32), Option<Vec<&'a mut Cell>>>,
}

impl<'a> ChunkWorker<'a> {
    pub fn new_from_chunk_ref(pos: &(i32, i32), chunks: &mut HashMap<(i32, i32), SplitChunk<'a>>) -> Self {
        // get center
        let chunk = match chunks.remove(pos).unwrap() {
            SplitChunk::Entire(chunk) => chunk,
            _ => panic!("Expected entire chunk for center"),
        };
        let surrounding_chunks = get_surrounding_chunks(chunks, pos.0, pos.1);

        Self {
            chunk,
            surrounding_chunks,
        }
    }

    pub fn update(&mut self) {
        // do basic falling over center chunk for testing

        for i in 0..self.chunk.cells.len() {
            let x = i % self.chunk.width as usize;
            let y = i / self.chunk.width as usize;

            let state_type = self.chunk.cells[i].get_state_type();
            match state_type {
                StateType::Empty(_) => {
                    // do nothing
                },
                StateType::SoftSolid(_) => {
                    self.downward_fall(&WorkerIndex {
                        chunk_rel: (0, 0),
                        idx: i,
                        x: x as i32,
                        y: y as i32,
                    });
                }
                _ => {
                    // do nothing
                }
            }

        }
    }

    fn get_cell_mut(&mut self, idx: WorkerIndex) -> Option<&mut Cell> {
        match idx.chunk_rel {
            (0, 0) => Some(&mut self.chunk.cells[idx.idx]),
            (x, y) => {
                match self.surrounding_chunks.get_mut(&(x, y)) {
                    None => None,
                    Some(chunk) => {
                        match chunk {
                            Some(chunk) => Some(&mut chunk[idx.idx]),
                            None => None,
                        }
                    }
                }
            },
        }
    }

    fn swap_cells(&mut self, c1: &WorkerIndex, c2: &WorkerIndex) {
        let (x1, y1) = c1.chunk_rel;

        // c1 should always be in the center chunk
        assert!(x1 == 0 && y1 == 0);

        match c2.chunk_rel {
            (0, 0) => {
                self.chunk.cells.swap(c1.idx, c2.idx);
            },
            (x, y) => {
                let c1 = &mut self.chunk.cells[c1.idx];
                let c2 = match self.surrounding_chunks.get_mut(&(x, y)) {
                    None => return,
                    Some(chunk) => {
                        match chunk {
                            None => return,
                            Some(chunk) => &mut chunk[c2.idx],
                        }
                    }
                };
                mem::swap(c1, c2);
            },
        }
    }

    // Gets the index of a relative chunk and index within that chunk
    fn get_worker_index(&self, x: i32, y: i32) -> WorkerIndex {
        if x >= 0 && x < self.chunk.width && y >= 0 && y < self.chunk.height {
            return WorkerIndex {
                chunk_rel: (0, 0),
                idx: get_index(x, y, self.chunk.width as i32),
                x,
                y,
            };
        } else {
            // if negative, we are dealing with a chunk to the left or below
            let x_c = if x < 0 { -1 } else if x >= self.chunk.width as i32 { 1 } else { 0 };
            let y_c = if y < 0 { -1 } else if y >= self.chunk.height as i32 { 1 } else { 0 };
            let (x, y) = (x % self.chunk.width as i32, y % self.chunk.height as i32);

            // Account for different sizes of borrowed chunks
            let x = match x_c {
                -1 => (self.chunk.width / 2) + x,
                1 => x,
                _ => x,
            };
            let y = match y_c {
                -1 => (self.chunk.height / 2) + y,
                1 => y,
                _ => y,
            };

            // Must have appropriate width depending on the borrowed chunk size
            let w = match (x_c, y_c) {
                (0, 0) => self.chunk.width,
                (1, 1) => self.chunk.width / 4,
                (-1, -1) => self.chunk.width / 4,
                (1, 0) => self.chunk.width / 2,
                (-1, 0) => self.chunk.width / 2,
                _ => self.chunk.width,
            };

            // width / 2 because we are only dealing with half chunks
            WorkerIndex {
                chunk_rel: (x_c, y_c),
                idx: get_index(x, y, w),
                x,
                y,
            }
        }
    }

    // Gets another cell based on position relative to a WorkerIndex
    fn get_other_cell(&mut self, idx: &WorkerIndex, dir: DirectionType) -> Option<&mut Cell> {
        let (x, y) = (idx.x, idx.y);
        let other_idx = dir.get_tuple_direction();
        let other_idx = (x + other_idx.0, y + other_idx.1);
        let other_idx = self.get_worker_index(other_idx.0, other_idx.1);
        self.get_cell_mut(other_idx)
    }

    fn downward_fall(&mut self, idx: &WorkerIndex) -> bool {
        let (x, y) = (idx.x, idx.y);
        let below = match self.get_other_cell(&idx, DirectionType::DOWN) {
            Some(cell) => cell,
            None => return false,
        };
        if below.get_type() == CellType::Empty {
            let other_idx = self.get_worker_index(x, y - 1);
            self.swap_cells(idx, &other_idx);
            return true;
        }
        false
    }
}

struct WorkerIndex {
    chunk_rel: (i32, i32),
    idx: usize, // idx within chunk

    // original x and y
    x: i32,
    y: i32,
}

fn get_index(x: i32, y: i32, width: i32) -> usize {
    (y * width + x) as usize
}

pub fn get_surrounding_chunks<'a>(
    chunks: &mut HashMap<(i32, i32), SplitChunk<'a>>,
    x: i32,
    y: i32,
) -> HashMap<(i32, i32), Option<Vec<&'a mut Cell>>> {
    let mut surrounding_chunks = HashMap::new();
    for i in -1..2 {
        for j in -1..2 {
            let pos = (x + i, y + j);
            let pos_rel = (i, j);
            if let Some(chunk) = chunks.get_mut(&pos) {
                match chunk {
                    SplitChunk::Entire(_) => { continue; },
                    SplitChunk::TopBottom(chunk) => {
                        if j == 1 {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[0]));
                        } else {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[1]));
                        }
                    },
                    SplitChunk::LeftRight(chunk) => {
                        if i == 1 {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[0]));
                        } else {
                            surrounding_chunks.insert(pos_rel, mem::take(&mut chunk[1]));
                        }
                    },
                    SplitChunk::Corners(_) => {
                        // unimplemented
                        continue;
                    },
                };
            }
        }
    }

    surrounding_chunks
}