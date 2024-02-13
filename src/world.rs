use std::sync::{Arc, Mutex};

use bevy::utils::hashbrown::HashMap;
use rand::Rng;

use crate::{cell::Cell, cell_types::{CellType, DirectionType}, chunk::PixelChunk};

pub struct PixelWorld {
    c_height: i32,
    c_width: i32,

    scale: f32,

    chunks: Vec<Arc<Mutex<PixelChunk>>>,

    chunks_lookup: HashMap<(i32, i32), Arc<Mutex<PixelChunk>>>
}

impl PixelWorld {

    pub fn new(t_width: i32, t_height: i32, scale: f32) -> Self {
        let mut new_world = PixelWorld {
            c_height: t_height / 8,
            c_width: t_width / 8,
            scale: scale,
            chunks: Vec::new(),
            chunks_lookup: HashMap::new()
        };

        // create chunks
        for x in 0..8 {
            for y in 0..8 {
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
        // bounds check -10..10
        if (x < -8 || x > 8) || (y < -8 || y > 8) {
            return None;
        }

        let chunk = Arc::new(Mutex::new(PixelChunk::new(self.c_width, self.c_height, x, y)));
        self.chunks.push(chunk.clone());
        self.chunks_lookup.insert((x, y), chunk.clone());
        Some(chunk)
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        // Check chunk
        match self.get_chunk(x, y) {
            Some(chunk) => {
                let chunk = chunk.lock().unwrap();
                chunk.in_bounds(x, y)
            },
            None => false
        }
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
        match self.get_chunk(x, y) {
            Some(chunk) => {
                let mut chunk = chunk.lock().unwrap();
                chunk.set_cell(x, y, cell);
            },
            None => {}
        }
    }

    fn move_cell(&self, x: i32, y: i32, xto: i32, yto: i32) {
        let loc_from = self.get_chunk_location(x, y);
        let loc_to = self.get_chunk_location(xto, yto);
        if (loc_from.0, loc_from.1) == (loc_to.0, loc_to.1) {
            match self.get_chunk(x, y) {
                Some(chunk) => {
                    let mut chunk = chunk.lock().unwrap();
                    let from_idx = chunk.get_index(x, y);
                    chunk.move_cell((None, from_idx), xto, yto);
                },
                None => {}
            }
        }
        else {
            // get both chunks
            let chunk_from = self.get_chunk(x, y);
            let chunk_to = self.get_chunk(xto, yto);
            if let Some(chunk_from) = chunk_from {
                let chunk_from_b = chunk_from.lock().unwrap();
                let from_idx = chunk_from_b.get_index(x, y);
                chunk_to.unwrap().lock().unwrap().move_cell((Some(chunk_from.clone()), from_idx), xto, yto);
            }

        }
    }

    // Update cells
    pub fn update(&mut self) {

        for chunk in self.chunks.iter() {
            let chunk = chunk.lock().unwrap();
            for x in 0..self.c_width {
                for y in 0..self.c_height {
                    let cell_movement = chunk.get_cell_2d(x, y).get_cell_movement();

                    if cell_movement.is_empty() {
                        continue;
                    }
                    else if cell_movement.intersects(DirectionType::DOWN) && self.move_down(x, y) {
                        continue;
                    }
                    else if cell_movement.intersects(DirectionType::LEFT | DirectionType::RIGHT) && self.move_side(x, y){
                        continue;
                    }
                    else if cell_movement.intersects(DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT) && self.move_diagonal(x, y) {
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

    fn move_down(&self, x: i32, y: i32) -> bool {
        let down = self.is_empty(x, y - 1);
        if down {
            self.move_cell(x, y, x, y - 1);
        }
        down
    }

    fn move_diagonal(&self, x: i32, y: i32) -> bool {
        let mut down_left = self.is_empty(x - 1, y - 1);
        let mut down_right = self.is_empty(x + 1, y - 1);
        if down_left && down_right {
            down_left = rand::thread_rng().gen_bool(0.5);
            down_right = !down_left;
        }

        if down_left {
            self.move_cell(x, y, x - 1, y - 1);
        }
        else if down_right {
            self.move_cell(x, y, x + 1, y - 1);
        }

        down_left || down_right
    }

    fn move_side(&self, x: i32, y: i32) -> bool {
        let mut left = self.is_empty(x - 1, y);
        let mut right = self.is_empty(x + 1, y);
        if left && right {
            left = rand::thread_rng().gen_bool(0.5);
            right = !left;
        }

        if left {
            self.move_cell(x, y, x - 1, y);
        }
        else if right {
            self.move_cell(x, y, x + 1, y);
        }

        left || right
    }
}