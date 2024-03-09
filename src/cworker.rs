use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::{cell_types::{should_move_density, CellType, DirectionType}, chunk::PixelChunk, world::PixelWorld};

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
                let cell = chunk.cells[chunk.get_index(x, y)];
                let cell_movement = cell.get_movement();
                let density = cell.get_density();

                let (x, y) = self.world.chunk_to_world_coords((chunk.pos_x, chunk.pos_y), (x, y));

                if cell_movement.intersects(DirectionType::DOWN) && self.move_down(x, y, density, &mut chunk) {
                    continue;
                } if cell_movement.intersects(DirectionType::UP) && self.move_up(x, y, density, &mut chunk) {
                    continue;
                } if cell_movement.intersects(DirectionType::LEFT | DirectionType::RIGHT) && self.move_side(x, y, density, &mut chunk){
                    continue;
                } if cell_movement.intersects(DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT) && self.move_diagonal_down(x, y, density, &mut chunk) {
                    continue;
                } if cell_movement.intersects(DirectionType::UP_LEFT | DirectionType::UP_RIGHT) && self.move_diagonal_up(x, y, density, &mut chunk) {
                    continue;
                }
            }
        }
    }

    fn move_down(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        if self.world.inside_chunk(chunk, (x, y - 1)) {
            if chunk.can_move_to(density, x, y - 1) {
                self.world.move_cell_same_chunk(x, y, x, y - 1, chunk);
                return true;
            }
        } else if self.world.chunk_exists_at_world_coord(x, y - 1) {
            if self.can_move_to_world(density, x, y - 1) {
                self.world.move_cell_diff_chunk(x, y, x, y - 1, chunk);
                return true;
            }
        }
        false
    }

    fn move_up(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        if self.world.inside_chunk(chunk, (x, y + 1)) {
            if chunk.can_move_to(density, x, y + 1) {
                self.world.move_cell_same_chunk(x, y, x, y + 1, chunk);
                return true;
            }
        } else if self.world.chunk_exists_at_world_coord(x, y + 1) {
            if self.world.is_empty(x, y + 1) {
                self.world.move_cell_diff_chunk(x, y, x, y + 1, chunk);
                return true;
            }
        }
        false
    }

    fn move_diagonal_down(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        let (mut down_left, down_left_inside) = {
            if self.world.inside_chunk(chunk, (x - 1, y - 1)) {
                (chunk.can_move_to(density, x - 1, y - 1), true)
            } else {
                (self.can_move_to_world(density, x - 1, y - 1), false)
            }
        };
        let (mut down_right, down_right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y - 1)) {
                (chunk.can_move_to(density, x + 1, y - 1), true)
            } else {
                (self.can_move_to_world(density, x + 1, y - 1), false)
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

    fn move_diagonal_up(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        let (mut up_left, up_left_inside) = {
            if self.world.inside_chunk(chunk, (x - 1, y + 1)) {
                (chunk.can_move_to(density, x - 1, y + 1), true)
            } else {
                (self.can_move_to_world(density, x - 1, y + 1), false)
            }
        };
        let (mut up_right, up_right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y + 1)) {
                (chunk.can_move_to(density, x + 1, y + 1), true)
            } else {
                (self.can_move_to_world(density, x + 1, y + 1), false)
            }
        };
        if up_left && up_right {
            up_left = rand::thread_rng().gen_bool(0.5);
            up_right = !up_left;
        }

        if up_left && up_left_inside {
            self.world.move_cell_same_chunk(x, y, x - 1, y + 1, chunk);
        }
        else if up_right && up_right_inside {
            self.world.move_cell_same_chunk(x, y, x + 1, y + 1, chunk);
        }
        else if up_left {
            self.world.move_cell_diff_chunk(x, y, x - 1, y + 1, chunk);
        }
        else if up_right {
            self.world.move_cell_diff_chunk(x, y, x + 1, y + 1, chunk);
        }

        up_left || up_right
    }

    fn move_side(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        let (mut left, left_inside) = {
            if self.world.inside_chunk(chunk, (x - 1, y)) {
                (chunk.can_move_to(density, x - 1, y), true)
            } else {
                (self.can_move_to_world(density, x - 1, y), false)
            }
        };
        let (mut right, right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y)) {
                (chunk.can_move_to(density, x + 1, y), true)
            } else {
                (self.can_move_to_world(density, x + 1, y), false)
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

    fn can_move_to_world(&self, density_from: f32, xto: i32, yto: i32) -> bool {
        match self.world.get_cell(xto, yto) {
            Some(cell) => cell.get_type() == CellType::Empty || should_move_density(density_from, cell.get_density()),
            None => false
        }
    }
}