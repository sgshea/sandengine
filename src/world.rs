use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use bevy::utils::HashMap;

use crate::{cell::Cell, chunk::PixelChunk, cworker::ChunkWorker};

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

    // Get locations of all chunks that are awake
    pub fn get_awake_chunk_locs(&self) -> Vec<(i32, i32)> {
        self.chunks.iter().filter_map(|chunk| {
            let chunk = chunk.lock().unwrap();
            if chunk.awake {
                Some((chunk.pos_x, chunk.pos_y))
            } else {
                None
            }
        }).collect()
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

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        match self.get_chunk(x, y) {
            Some(chunk) => {
                let chunk = chunk.lock().unwrap();
                chunk.is_empty(x, y)
            },
            None => false
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.c_width * self.chunks_x && y >= 0 && y < self.c_height * self.chunks_y
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Option<Cell> {
        if x < 0 || x >= self.c_width * self.chunks_x || y < 0 || y >= self.c_height * self.chunks_y {
            return None;
        }
        match self.get_chunk(x, y) {
            Some(chunk) => {
                let chunk = chunk.lock().unwrap();
                Some(chunk.get_cell_2d(x, y).clone())
            },
            None => None
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

    pub fn move_cell_diff_chunk(&self, x: i32, y: i32, xto: i32, yto: i32, chunk: &mut PixelChunk) {
        let chunk_from = self.get_chunk(x, y);
        match self.get_chunk(xto, yto) {
            Some(chunk_to) => {
                let mut chunk_to_m = chunk_to.lock().unwrap();
                let from_idx = chunk.get_index(x, y);
                let to_idx = chunk_to_m.get_index(xto, yto);
                chunk_to_m.changes.push((chunk_from, from_idx, to_idx));
                chunk_to_m.awake = true;
            },
            None => {}
        }
    }

    pub fn chunk_to_world_coords(&self, chunk_pos: (i32, i32), cell_pos: (i32, i32)) -> (i32, i32) {
        (chunk_pos.0 * self.c_width + cell_pos.0, chunk_pos.1 * self.c_height + cell_pos.1)
    }

    pub fn inside_chunk(&self, chunk: &PixelChunk, world_coord: (i32, i32)) -> bool {
        (chunk.pos_x, chunk.pos_y) == self.get_chunk_location(world_coord.0, world_coord.1)
    }

    pub fn chunk_exists_at_world_coord(&self, x: i32, y: i32) -> bool {
        self.get_chunk(x, y).is_some()
    }

    // Update cells
    pub fn update(&mut self) {

        // update in checkerboard based on position
        self.chunks.par_iter().for_each(|chunk| {
            let pos = chunk.lock().unwrap().get_pos();
            if (pos.0 + pos.1) % 2 == 0 {
                ChunkWorker::new(self, chunk.clone()).update_chunk();
            }
        });
        // update rest
        self.chunks.par_iter().for_each(|chunk| {
            let pos = chunk.lock().unwrap().get_pos();
            if (pos.0 + pos.1) % 2 != 0 {
                ChunkWorker::new(self, chunk.clone()).update_chunk();
            }
        });

        // commit changes
        self.chunks.par_iter().for_each(|chunk| {
            let pos = chunk.lock().unwrap().get_pos();
            if (pos.0 + pos.1) % 2 == 0 {
                let mut chunk = chunk.lock().unwrap();
                chunk.commit_cells();
            }
        });
        // commit rest
        self.chunks.par_iter().for_each(|chunk| {
            let pos = chunk.lock().unwrap().get_pos();
            if (pos.0 + pos.1) % 2 != 0 {
                let mut chunk = chunk.lock().unwrap();
                chunk.commit_cells();
            }
        });
    }

}