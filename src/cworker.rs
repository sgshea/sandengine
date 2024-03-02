use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::{cell_types::DirectionType, chunk::PixelChunk, world::PixelWorld};

pub struct ChunkWorker<'a> {
    world: &'a PixelWorld,
    chunk: Arc<Mutex<PixelChunk>>,
}

impl ChunkWorker<'_> {
    pub fn new(world: &PixelWorld, chunk: Arc<Mutex<PixelChunk>>) -> ChunkWorker {
        ChunkWorker { world, chunk }
    }

    pub fn update_chunk(&self) {
        let mut chunk = self.chunk.lock().unwrap();
        for x in 0..chunk.width {
            for y in 0..chunk.height {
                let cell_movement = chunk.get_cell_2d(x, y).get_cell_movement();

                let (x, y) = self.world.chunk_to_world_coords((chunk.pos_x, chunk.pos_y), (x, y));

                if cell_movement.intersects(DirectionType::DOWN) && self.move_down(x, y, &mut chunk) {
                    continue;
                } if cell_movement.intersects(DirectionType::LEFT | DirectionType::RIGHT) && self.move_side(x, y, &mut chunk){
                    continue;
                } if cell_movement.intersects(DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT) && self.move_diagonal(x, y, &mut chunk) {
                    continue;
                }
            }
        }
    }

    fn move_down(&self, x: i32, y: i32, chunk: &mut PixelChunk) -> bool {
        if self.world.inside_chunk(chunk, (x, y - 1)) {
            if chunk.is_empty(x, y - 1) {
                self.world.move_cell_same_chunk(x, y, x, y - 1, chunk);
                return true;
            }
        } else if self.world.chunk_exists_at_world_coord(x, y - 1) {
            if self.world.is_empty(x, y - 1) {
                self.world.move_cell_diff_chunk(x, y, x, y - 1, chunk);
                return true;
            }
        }
        false
    }

    fn move_diagonal(&self, x: i32, y: i32, chunk: &mut PixelChunk) -> bool {
        let (mut down_left, down_left_inside) = {
            if self.world.inside_chunk(chunk, (x - 1, y - 1)) {
                (chunk.is_empty(x - 1, y - 1), true)
            } else {
                (self.world.is_empty(x - 1, y - 1), false)
            }
        };
        let (mut down_right, down_right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y - 1)) {
                (chunk.is_empty(x + 1, y - 1), true)
            } else {
                (self.world.is_empty(x + 1, y - 1), false)
            }
        };
        if down_left && down_right {
            down_left = rand::thread_rng().gen_bool(0.5);
            down_right = !down_left;
        }

        if down_left && down_left_inside {
            self.world.move_cell_same_chunk(x, y, x - 1, y - 1, chunk);
        }
        else if down_right && down_right_inside {
            self.world.move_cell_same_chunk(x, y, x + 1, y - 1, chunk);
        }
        else if down_left {
            self.world.move_cell_diff_chunk(x, y, x - 1, y - 1, chunk);
        }
        else if down_right {
            self.world.move_cell_diff_chunk(x, y, x + 1, y - 1, chunk);
        }

        down_left || down_right
    }

    fn move_side(&self, x: i32, y: i32, chunk: &mut PixelChunk) -> bool {
        let (mut left, left_inside) = {
            if self.world.inside_chunk(chunk, (x - 1, y)) {
                (chunk.is_empty(x - 1, y), true)
            } else {
                (self.world.is_empty(x - 1, y), false)
            }
        };
        let (mut right, right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y)) {
                (chunk.is_empty(x + 1, y), true)
            } else {
                (self.world.is_empty(x + 1, y), false)
            }
        };
        if left && right {
            left = rand::thread_rng().gen_bool(0.5);
            right = !left;
        }

        if left && left_inside {
            self.world.move_cell_same_chunk(x, y, x - 1, y, chunk);
        }
        else if right && right_inside {
            self.world.move_cell_same_chunk(x, y, x + 1, y, chunk);
        }
        else if left {
            self.world.move_cell_diff_chunk(x, y, x - 1, y, chunk);
        }
        else if right {
            self.world.move_cell_diff_chunk(x, y, x + 1, y, chunk);
        }

        left || right
    }
}