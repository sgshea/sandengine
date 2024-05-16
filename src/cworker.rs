use std::{cell, mem};

use bevy::{math::Vec2, utils::hashbrown::HashMap};
use rand::Rng;

use crate::{cell::Cell, cell_types::{CellType, DirectionType, StateType}, chunk::{PixelChunk, SplitChunk}};

// Maximum Speed constant
const MAX_SPEED: f32 = 8.0;
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
            if self.iter_dir && y % 2 == 0 {
                for x in 0..self.chunk.width {
                    self.update_cell(x, y);
                }
            } else {
                for x in (0..self.chunk.width).rev() {
                    self.update_cell(x, y);
                }
            }
        }
    }

    fn update_cell(&mut self, x: i32, y: i32) {
        let state_type = self.chunk.cells[get_index(x, y, self.chunk.width as i32)].get_state_type();
        let new_state = self.chunk.next_cells[get_index(x, y, self.chunk.width as i32)].get_state_type();
        if state_type != new_state {
            return;
        }
        match state_type {
            StateType::Empty(_) => {
                // do nothing
                return;
            },
            StateType::SoftSolid(_) => {
                let idx = self.get_worker_index(x, y);
                self.add_gravity(&idx);
                if self.downward_fall(&idx) {
                    return;
                }
                if self.apply_velocity(&idx) {
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
                self.chunk.next_cells.swap(c1.idx, c2.idx);
            },
            (x, y) => {
                // get both current and next of the chunk
                let next_chunk = self.surrounding_next.get_mut(&(x, y)).unwrap();

                let c1_c = self.chunk.next_cells[c1.idx].clone();
                self.chunk.next_cells[c1.idx] = next_chunk.as_mut().unwrap()[c2.idx].clone();
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

    fn get_cell_next(&self, x: i32, y: i32) -> Option<&Cell> {
        let idx = self.get_worker_index(x, y);
        match idx.chunk_rel {
            (0, 0) => Some(&self.chunk.next_cells[idx.idx]),
            (x, y) => {
                match self.surrounding_next.get(&(x, y)) {
                    None => None,
                    Some(chunk) => {
                        match chunk {
                            None => None,
                            Some(chunk) => {
                                // let cell = &chunk[other_idx.idx];
                                // println!("{} {} ({}, {}), ({:?})", idx.idx, other_idx.idx, other_idx.x, other_idx.y, cell.get_type());
                                Some(&chunk[idx.idx])
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

        let cell = &self.chunk.next_cells[idx.idx];

        let down_next = self.get_other_cell_next(&idx, DirectionType::DOWN);
        // we want to swap down if the cell below has a lower density AND the one below that has a higher density (else want to become particle like)
        if down_next.is_some_and(|t| t.get_density() < cell.get_density()) {
            let new_idx = self.get_worker_index(x, y - 1);
            let down_next_2 = self.get_other_cell_next(&new_idx, DirectionType::DOWN);
            if down_next_2.is_some_and(|t| t.get_density() >= cell.get_density()) {
                self.swap_cells(idx, &new_idx);
                return true;
            }
        }
        false
    }

    fn down_side(&mut self, idx: &WorkerIndex) -> bool {
        let (x, y) = (idx.x, idx.y);
        let left = self.get_other_cell_next(&idx, DirectionType::DOWN_LEFT);
        let right = self.get_other_cell_next(&idx, DirectionType::DOWN_RIGHT);
        // get types and make sure they are empty and has not been updated
        let density = self.chunk.next_cells[idx.idx].get_density();
        let mut move_left = left.is_some_and(|t| t.get_density() < density);
        let mut move_right = right.is_some_and(|t| t.get_density() < density);
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

    fn add_gravity(&mut self, idx: &WorkerIndex) {
        // check below exists
        if self.get_other_cell_next(idx, DirectionType::DOWN).is_none() {
            self.chunk.next_cells[idx.idx].velocity.y = 0.;
            return;
        }
        let below_cell = self.get_other_cell_next(idx, DirectionType::DOWN);
        let below_density = below_cell.unwrap().get_density();
        let below_velocity = below_cell.unwrap().velocity;

        let cell = &mut self.chunk.next_cells[idx.idx];
        // Clamp current velocity
        cell.velocity = cell.velocity.clamp(Vec2::new(-MAX_SPEED, -MAX_SPEED), Vec2::new(MAX_SPEED, MAX_SPEED));

        const LIMIT: f32 = 3.;
        if below_density < cell.get_density() && cell.velocity.y < LIMIT {
            cell.velocity.y += 0.7;
        } else {
            if below_velocity.y.abs() < 0.5 {
                // deflection into x direction
                if cell.velocity.x == 0. {
                    // 50% chance to go left or right
                    if rand::thread_rng().gen_bool(0.5) {
                        cell.velocity.x += cell.velocity.y / 5.;
                    } else {
                        cell.velocity.x -= cell.velocity.y / 5.;
                    }
                } else {
                    if cell.velocity.x < 0. {
                        cell.velocity.x -= (cell.velocity.y / 5.).abs();
                    } else {
                        cell.velocity.x += (cell.velocity.y / 5.).abs();
                    }
                }
                // set y velocity to 0
                cell.velocity.y = 0.;
            }
        }
    }

    fn apply_velocity(&mut self, idx: &WorkerIndex) -> bool {
        let cell = &mut self.chunk.next_cells[idx.idx];
        let cell_density = cell.get_density();

        let vector_length = cell.velocity.length();

        // No significant velocity
        if vector_length < 0.5 {
            return false;
        }

        // clamp to half chunk length (assumes square chunks)
        // ensuring that it does not try to move outside of what the worker has access to
        let max_velocity = self.chunk.width as f32 / 2.;
        cell.velocity.x = cell.velocity.x.clamp(-max_velocity, max_velocity);
        cell.velocity.y = cell.velocity.y.clamp(-max_velocity, max_velocity);

        // reset x dir
        if cell.get_type() == CellType::Sand && cell.velocity.x.abs() < 1. {
            cell.velocity.x = 0.;
        }

        let (f_x, f_y) = (cell.velocity.x / vector_length, cell.velocity.y / vector_length);

        // No significant force
        if f_x == 0. && f_y == 0. {
            return false;
        }

        // Moving elements to furthest position possible
        let (mut max_x, mut max_y) = (idx.x as f32, idx.y as f32);
        let (x, y) = (idx.x as f32, idx.y as f32);
        let drag = 0.9;
        for i in 1..=vector_length.round() as i32 {
            // calculate index
            let (x, y) = ((x as f32 - (f_x * i as f32)).round() as i32, (y as f32 - (f_y * i as f32)).round() as i32);

            // trying to move here
            let other_cell = self.get_cell_next(x, y);

            // cell is none or solid, cannot move futher
            if other_cell.is_none() || matches!(other_cell.unwrap().get_state_type(), StateType::HardSolid(_)) {
                if i == 1 || other_cell.is_none() {
                    // immediately stoped
                    let cell = &mut self.chunk.next_cells[idx.idx];
                    cell.velocity = Vec2::ZERO;
                    return false;
                } else {
                    if max_x != idx.x as f32 || max_y != idx.y as f32 {
                        // move to max_x, max_y
                        let cell = &mut self.chunk.next_cells[idx.idx];
                        cell.velocity *= drag;
                        let new_idx = self.get_worker_index(max_x as i32, max_y as i32);
                        self.swap_cells(idx, &new_idx);
                        return true;
                    } else {
                        // stop
                        let cell = &mut self.chunk.next_cells[idx.idx];
                        cell.velocity = Vec2::ZERO;
                        return false;
                    }
                }
            } else {
                if other_cell.unwrap().get_density() < cell_density {
                    // new furthest position
                    (max_x, max_y) = (x as f32, y as f32);
                }
            }

            // No solid cells found and at maximum length
            if i == vector_length.round() as i32 {
                if max_x != idx.x as f32 || max_y != idx.y as f32 {
                    // move to max_x, max_y
                    let cell = &mut self.chunk.next_cells[idx.idx];
                    cell.velocity *= drag;
                    let new_idx = self.get_worker_index(max_x as i32, max_y as i32);
                    self.swap_cells(idx, &new_idx);
                    return true;
                } else {
                    // stop
                    let cell = &mut self.chunk.next_cells[idx.idx];
                    cell.velocity = Vec2::ZERO;
                    return false;
                }
            }
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