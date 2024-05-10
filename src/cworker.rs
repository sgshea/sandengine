use std::{cell, mem};

use bevy::utils::hashbrown::HashMap;
use rand::Rng;

use crate::{cell::Cell, cell_types::{CellType, DirectionType, StateType}, chunk::{PixelChunk, SplitChunk}};

pub struct ChunkWorker<'a> {
    chunk: &'a mut PixelChunk,
    surrounding_current: HashMap<(i32, i32), Option<Vec<&'a mut Cell>>>,
    surrounding_next: HashMap<(i32, i32), Option<Vec<&'a mut Cell>>>,
    iter_dir: bool,
}

impl<'a> ChunkWorker<'a> {
    pub fn new_from_chunk_ref(pos: &(i32, i32), current: &mut HashMap<(i32, i32), SplitChunk<'a>>, next: &mut HashMap<(i32, i32), SplitChunk<'a>>, iter_dir: bool) -> Self {
        // get center
        let chunk = match current.remove(pos).unwrap() {
            SplitChunk::Entire(chunk) => chunk,
            _ => panic!("Expected entire chunk for center"),
        };
        let surrounding_current = get_surrounding_chunks(current, pos.0, pos.1);
        let surrounding_next = get_surrounding_chunks(next, pos.0, pos.1);

        Self {
            chunk,
            surrounding_current,
            surrounding_next,
            iter_dir,
        }
    }

    pub fn update(&mut self) {
        for y in 0..self.chunk.height {
            for x in 0..self.chunk.width {
                self.update_cell(x, y);
            }
        }
    }

    fn update_cell(&mut self, x: i32, y: i32) {
        let state_type = self.chunk.get_cell_2d(x, y).get_state_type();
        match state_type {
            StateType::Empty(_) => {
                // do nothing
                return;
            },
            StateType::SoftSolid(_) => {
                let idx = self.get_worker_index(x, y);
                if self.downward_fall(&idx) {
                    return;
                }
                if self.down_side(&idx) {
                    return;
                }
            }
            _ => {
                // do nothing
            }
        }
    }

    fn swap_cells(&mut self, c1: &WorkerIndex, c2: &WorkerIndex) {
        let (x1, y1) = c1.chunk_rel;

        // c1 should always be in the center chunk
        assert!(x1 == 0 && y1 == 0);

        match c2.chunk_rel {
            (0, 0) => {
                let c1_c = self.chunk.cells[c1.idx].clone();
                self.chunk.next_cells[c1.idx] = self.chunk.cells[c2.idx].clone();
                self.chunk.next_cells[c2.idx] = c1_c;
            },
            (x, y) => {
                // get both current and next of the chunk
                let cur_chunk = self.surrounding_current.get_mut(&(x, y)).unwrap();
                let next_chunk = self.surrounding_next.get_mut(&(x, y)).unwrap();

                let c1_c = self.chunk.cells[c1.idx].clone();
                self.chunk.next_cells[c1.idx] = cur_chunk.as_ref().unwrap()[c2.idx].clone();
                *next_chunk.as_mut().unwrap()[c2.idx] = c1_c;
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
            // if self.chunk.pos_y != 0 {
            //     println!("{} {} ({} {})", x, y, self.chunk.pos_x, self.chunk.pos_y);
            // }
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

    fn get_other_cell(&self, idx: &WorkerIndex, dir: DirectionType) -> Option<&Cell> {
        let (x, y) = (idx.x, idx.y);
        let other_idx = dir.get_tuple_direction();
        let other_idx = (x + other_idx.0, y + other_idx.1);
        let other_idx = self.get_worker_index(other_idx.0, other_idx.1);
        match other_idx.chunk_rel {
            (0, 0) => Some(&self.chunk.cells[other_idx.idx]),
            (x, y) => {
                match self.surrounding_current.get(&(x, y)) {
                    None => None,
                    Some(chunk) => {
                        match chunk {
                            None => None,
                            Some(chunk) => {
                                // let cell = &chunk[other_idx.idx];
                                // println!("{} {} ({}, {}), ({:?})", idx.idx, other_idx.idx, other_idx.x, other_idx.y, cell.get_type());
                                Some(&chunk[other_idx.idx])
                            }
                        }
                    }
                }
            },
        }
    }

    fn get_other_cell_next(&self, idx: &WorkerIndex, dir: DirectionType) -> Option<&Cell> {
        let (x, y) = (idx.x, idx.y);
        let other_idx = dir.get_tuple_direction();
        let other_idx = (x + other_idx.0, y + other_idx.1);
        let other_idx = self.get_worker_index(other_idx.0, other_idx.1);
        match other_idx.chunk_rel {
            (0, 0) => Some(&self.chunk.next_cells[other_idx.idx]),
            (x, y) => {
                match self.surrounding_next.get(&(x, y)) {
                    None => None,
                    Some(chunk) => {
                        match chunk {
                            None => None,
                            Some(chunk) => {
                                // let cell = &chunk[other_idx.idx];
                                // println!("{} {} ({}, {}), ({:?})", idx.idx, other_idx.idx, other_idx.x, other_idx.y, cell.get_type());
                                Some(&chunk[other_idx.idx])
                            }
                        }
                    }
                }
            },
        }
    }

    fn downward_fall(&mut self, idx: &WorkerIndex) -> bool {
        let (x, y) = (idx.x, idx.y);

        let down = self.get_other_cell(&idx, DirectionType::DOWN);
        let down_next = self.get_other_cell_next(&idx, DirectionType::DOWN);
        if let Some(cell) = down {
            if cell.get_type() == CellType::Empty && (down_next.is_none() || down_next.is_some_and(|t| t.get_type() == CellType::Empty)) {
                self.swap_cells(idx, &self.get_worker_index(x, y - 1));
                return true;
            }
        }

        false
    }

    fn down_side(&mut self, idx: &WorkerIndex) -> bool {
        let (x, y) = (idx.x, idx.y);
        let left = self.get_other_cell(&idx, DirectionType::DOWN_LEFT);
        let right = self.get_other_cell(&idx, DirectionType::DOWN_RIGHT);
        // get types and make sure they are empty and has not been updated
        let mut move_left = left.is_some_and(|t| t.get_type() == CellType::Empty);
        let mut move_right = right.is_some_and(|t| t.get_type() == CellType::Empty);
        if move_left && move_right {
            // choose 50/50
            move_left = rand::thread_rng().gen_bool(0.5);
            move_right = !move_left;
        }

        if move_left {
            let other_idx = self.get_worker_index(x - 1, y - 1);
            self.swap_cells(idx, &other_idx);
            return true;
        }
        else if move_right {
            let other_idx = self.get_worker_index(x + 1, y - 1);
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