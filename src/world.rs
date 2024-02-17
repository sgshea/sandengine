use std::sync::{Arc, Mutex};

use bevy::utils::hashbrown::HashMap;
use rand::Rng;

use crate::{cell::Cell, cell_types::DirectionType, chunk::PixelChunk};

pub struct PixelWorld {
    c_height: i32,
    c_width: i32,

    chunks_x: i32,
    chunks_y: i32,

    chunks: Vec<Arc<Mutex<PixelChunk>>>,

    chunks_lookup: HashMap<(i32, i32), Arc<Mutex<PixelChunk>>>
}

impl PixelWorld {

    pub fn new(t_width: i32, t_height: i32, chunks_x: i32, chunks_y: i32) -> Self {
        let mut new_world = PixelWorld {
            c_height: t_height / chunks_x,
            c_width: t_width / chunks_y,
            chunks_x,
            chunks_y,
            chunks: Vec::new(),
            chunks_lookup: HashMap::new()
        };

        // create chunks
        for x in 0..chunks_x {
            for y in 0..chunks_y {
                new_world.create_chunk(x, y);
            }
        }

        new_world
    }

    pub fn get_chunk_direct(&self, x: i32, y: i32) -> Option<Arc<Mutex<PixelChunk>>> {
        self.chunks_lookup.get(&(x, y)).map(|c| c.clone())
    }

    pub fn get_chunk_location(&self, x: i32, y: i32) -> (i32, i32) {
        (x / self.c_width, y / self.c_height)
    }

    pub fn get_chunk(&self, x: i32, y: i32) -> Option<Arc<Mutex<PixelChunk>>> {
        let (cx, cy) = self.get_chunk_location(x, y);
        self.get_chunk_direct(cx, cy)
    }

    fn create_chunk(&mut self, x: i32, y: i32) -> Option<Arc<Mutex<PixelChunk>>> {
        let chunk = Arc::new(Mutex::new(PixelChunk::new(self.c_width, self.c_height, x, y)));
        self.chunks.push(chunk.clone());
        self.chunks_lookup.insert((x, y), chunk.clone());
        Some(chunk)
    }

    fn is_empty(&self, x: i32, y: i32) -> bool {
        match self.get_chunk(x, y) {
            Some(chunk) => {
                let chunk = chunk.lock().unwrap();
                chunk.is_empty(x, y)
            },
            None => false
        }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        match self.get_chunk(x, y) {
            Some(chunk) => {
                let chunk = chunk.lock().unwrap();
                chunk.get_cell_2d(x, y).clone()
            },
            None => Cell::empty()
        }
    }

    pub fn set_cell(&self, x: i32, y: i32, cell: Cell) {
        // Check if the cell is in bounds
        if x < 0 || x >= self.c_width * self.chunks_x || y < 0 || y >= self.c_height * self.chunks_y {
            return;
        }

        match self.get_chunk(x, y) {
            Some(chunk) => {
                let mut chunk = chunk.lock().unwrap();
                chunk.set_cell(x, y, cell);
            },
            None => {}
        }
    }

    fn move_cell_diff_chunk(&self, x: i32, y: i32, xto: i32, yto: i32, chunk: &mut PixelChunk) {
        let chunk_to = self.get_chunk(xto, yto);
        match self.get_chunk(x, y) {
            Some(chunk_from) => {
                let mut chunk_from = chunk_from.lock().unwrap();
                let from_idx = chunk_from.get_index(x, y);
                chunk.changes.push((chunk_to, from_idx, chunk.get_index(xto, yto)));
            },
            None => {}
        }
    }

    fn move_cell_same_chunk(&self, x: i32, y: i32, xto: i32, yto: i32, chunk: &mut PixelChunk) {
        let from_idx = chunk.get_index(x, y);
        chunk.changes.push((None, from_idx, chunk.get_index(xto, yto)));
    }

    fn chunk_to_world_coords(&self, chunk_pos: (i32, i32), cell_pos: (i32, i32)) -> (i32, i32) {
        (chunk_pos.0 * self.c_width + cell_pos.0, chunk_pos.1 * self.c_height + cell_pos.1)
    }

    fn inside_chunk(&self, chunk: &PixelChunk, world_coord: (i32, i32)) -> bool {
        (chunk.pos_x, chunk.pos_y) == self.get_chunk_location(world_coord.0, world_coord.1)
    }

    // Update cells
    pub fn update(&mut self) {

        for chunk in self.chunks.iter() {
            let mut chunk = chunk.lock().unwrap();
            let mut chunk = chunk.lock().unwrap();
            for x in 0..self.c_width {
                for y in 0..self.c_height {
                    let cell_movement = chunk.get_cell_2d(x, y).get_cell_movement();

                    let (x, y) = self.chunk_to_world_coords((chunk.pos_x, chunk.pos_y), (x, y));

                    let (x, y) = self.chunk_to_world_coords((chunk.pos_x, chunk.pos_y), (x, y));

                    if cell_movement.is_empty() {
                        continue;
                    }
                    else if cell_movement.intersects(DirectionType::DOWN) && self.move_down(x, y, &mut chunk) {
                    else if cell_movement.intersects(DirectionType::DOWN) && self.move_down(x, y, &mut chunk) {
                        continue;
                    }
                    else if cell_movement.intersects(DirectionType::LEFT | DirectionType::RIGHT) && self.move_side(x, y, &mut chunk){
                        continue;
                    }
                    else if cell_movement.intersects(DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT) && self.move_diagonal(x, y, &mut chunk) {
                        continue;
                    }
                }
            }
        }

        for chunk in self.chunks.iter() {
            let mut chunk = chunk.lock().unwrap();
            chunk.commit_cells();
        }
    }

    fn move_down(&self, x: i32, y: i32, chunk: &mut PixelChunk) -> bool {
        if self.inside_chunk(chunk, (x, y)) {
            if chunk.is_empty(x, y - 1) {
                self.move_cell_same_chunk(x, y, x, y - 1, chunk);
                return true;
            }
            return false;
        } else {
            if self.is_empty(x, y - 1) {
                self.move_cell_diff_chunk(x, y, x, y - 1, chunk);
                return true;
            }
            return false;
        }
        false
    }

    fn move_diagonal(&self, x: i32, y: i32, chunk: &mut PixelChunk) -> bool {
        let (mut down_left, down_left_inside) = {
            if self.inside_chunk(chunk, (x - 1, y - 1)) {
                (chunk.is_empty(x - 1, y - 1), true)
            } else {
                (self.is_empty(x - 1, y - 1), false)
            }
        };
        let (mut down_right, down_right_inside) = {
            if self.inside_chunk(chunk, (x + 1, y - 1)) {
                (chunk.is_empty(x + 1, y - 1), true)
            } else {
                (self.is_empty(x + 1, y - 1), false)
            }
        };
        if down_left && down_right {
            down_left = rand::thread_rng().gen_bool(0.5);
            down_right = !down_left;
        }

        if down_left && down_left_inside {
            self.move_cell_same_chunk(x, y, x - 1, y - 1, chunk);
        }
        else if down_right && down_right_inside {
            self.move_cell_same_chunk(x, y, x + 1, y - 1, chunk);
        }
        else if down_left {
            self.move_cell_diff_chunk(x, y, x - 1, y - 1, chunk);
        }
        else if down_right {
            self.move_cell_diff_chunk(x, y, x + 1, y - 1, chunk);
        }

        down_left || down_right
    }

    fn move_side(&self, x: i32, y: i32, chunk: &mut PixelChunk) -> bool {
        if self.inside_chunk(chunk, (x - 1, y)) && self.inside_chunk(chunk, (x + 1, y)) {
            let mut left = chunk.is_empty(x - 1, y);
            let mut right = chunk.is_empty(x + 1, y);
            if left && right {
                left = rand::thread_rng().gen_bool(0.5);
                right = !left;
            }

            if left {
                self.move_cell_same_chunk(x, y, x - 1, y, chunk);
            }
            else if right {
                self.move_cell_same_chunk(x, y, x + 1, y, chunk);
            }

            return left || right;
        }
        else if self.chunk_exists_at_world_coord(x - 1, y) && self.chunk_exists_at_world_coord(x + 1, y) {
            let mut left = self.is_empty(x - 1, y);
            let mut right = self.is_empty(x + 1, y);
            if left && right {
                left = rand::thread_rng().gen_bool(0.5);
                right = !left;
            }

            if left {
                self.move_cell_diff_chunk(x, y, x - 1, y, chunk);
            }
            else if right {
                self.move_cell_diff_chunk(x, y, x + 1, y, chunk);
            }

            return left || right;
        }
        false
    }
}