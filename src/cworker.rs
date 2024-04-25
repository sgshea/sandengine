use std::sync::{Arc, Mutex, MutexGuard};

use crate::{cell::Cell, cell_effect::*, cell_types::{should_move_density, CellType, DirectionType}, chunk::PixelChunk, world::PixelWorld};

pub struct ChunkWorker<'a> {
    pub world: &'a PixelWorld,
    // Chunk to work on
    chunk: Arc<Mutex<PixelChunk>>,
}

impl ChunkWorker<'_> {
    pub fn new(world: &PixelWorld, chunk: Arc<Mutex<PixelChunk>>) -> ChunkWorker {
        ChunkWorker { world, chunk }
    }

    pub fn update_chunk(&self) {
        let mut chunk = self.chunk.lock().unwrap();
        if !chunk.awake {
            chunk.awake = chunk.awake_next;
            return;
        }

        // movement
        let movement_effect = MovementEffect;
        for x in 0..chunk.width {
            for y in 0..chunk.height {
                let (x, y) = self.world.chunk_to_world_coords((chunk.pos_x, chunk.pos_y), (x, y));
                let surrounding_cells = self.get_surrounding_cells(x, y, &chunk);

                movement_effect.apply(chunk.cells[chunk.get_index(x, y)], (x, y), surrounding_cells, self, &mut chunk);
            }
        }

        chunk.awake_next = if chunk.changes.is_empty() { false } else { true };
        chunk.awake = chunk.awake_next;
    }

    fn get_surrounding_cells(&self, x: i32, y: i32, chunk: &PixelChunk) -> Vec<(DirectionType, Cell)> {
        let mut cells = Vec::new();
        for direction in DirectionType::all() {
            let (dx, dy) = direction.get_tuple_direction();
            let x = x + dx;
            let y = y + dy;
            if chunk.in_bounds_world(x, y) {
                let idx = chunk.get_index(x, y);
                cells.push((direction, chunk.cells[idx].clone()));
            }
            // } else if let Some(chunk) = self.world.get_chunk(x, y) {
            //     match chunk.lock() {
            //         Ok(chunk) => {
            //             let idx = chunk.get_index(x, y);
            //             cells.push((direction, chunk.cells[idx].clone()));
            //         },
            //         Err(_) => {}
            //     }
            // }
        }
        cells
    }

    // Abstraction to move a cell, handles going to different chunks
    pub fn move_cell(&self, original_loc: (i32, i32), direction: DirectionType, chunk: &mut PixelChunk ) {
        // Get tuple direction
        let (dx, dy) = direction.get_tuple_direction();
        // Get new location
        let new_loc = (original_loc.0 + dx, original_loc.1 + dy);
        if self.world.inside_chunk(&chunk, new_loc) {
            self.move_cell_same_chunk(original_loc.0, original_loc.1, new_loc.0, new_loc.1, chunk);
            // Wake up surrounding chunks
            self.wake_chunk_helper(chunk, new_loc.0, new_loc.1);
        } else {
            self.world.move_cell_diff_chunk(original_loc.0, original_loc.1, new_loc.0, new_loc.1, chunk);
            // Wake up surrounding chunks
            self.wake_chunk_helper(chunk, new_loc.0, new_loc.1);
        }
    }

    // Expects chunk pos
    fn wake_chunk_in_direction(&self, x: i32, y: i32, direction: DirectionType) {
        let (x, y) = match direction {
            DirectionType::UP => (x, y + 1),
            DirectionType::DOWN => (x, y - 1),
            DirectionType::LEFT => (x - 1, y),
            DirectionType::RIGHT => (x + 1, y),
            // Trying to lock diagonals causes a deadlock
            _ => return
        };
        if let Some(chunk) = self.world.get_chunk_direct(x, y) {
            let mut chunk = chunk.lock().unwrap();
            chunk.awake= true;
        }
    }

    fn wake_chunk_helper(&self, chunk: &PixelChunk, x: i32, y: i32) {
        let (x, y) = world_to_chunk_coords(chunk, x, y);

        if y >= chunk.height - 3 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::UP);
        } if x <= 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::LEFT);
        } if x >= chunk.width - 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::RIGHT);
        } if x <= 2 && y >= chunk.height - 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::UP_LEFT);
        } if x >= chunk.width - 2 && y >= chunk.height - 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::UP_RIGHT);
        } if x <= 2 && y <= 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::DOWN_LEFT);
        } if x >= chunk.width - 1 && y <= 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::DOWN_RIGHT);
        } if y <= 1 {
            self.wake_chunk_in_direction(chunk.pos_x, chunk.pos_y, DirectionType::DOWN);
        }
    }
    pub fn move_cell_same_chunk(&self, x: i32, y: i32, xto: i32, yto: i32, chunk: &mut PixelChunk) {
        let from_idx = chunk.get_index(x, y);
        chunk.changes.push((None, from_idx, chunk.get_index(xto, yto)));
    }
}

#[inline]
fn world_to_chunk_coords(chunk: &PixelChunk, x: i32, y: i32) -> (i32, i32) {
    (x % chunk.width, y % chunk.height)
}