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
        if !chunk.awake {
            chunk.awake = chunk.awake_next;
            return;
        }

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

        chunk.awake_next = if chunk.changes.is_empty() { false } else { true };
        chunk.awake = chunk.awake_next;
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

        if y >= chunk.height - 1 {
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

    fn move_down(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        if self.world.inside_chunk(chunk, (x, y - 1)) {
            if can_move_to(chunk, density, x, y - 1) {
                // Awake chunk above if at top of this chunk
                self.wake_chunk_helper(chunk, x, y);

                self.move_cell_same_chunk(x, y, x, y - 1, chunk);
                return true;
            }
        } else if self.world.chunk_exists_at_world_coord(x, y - 1) {
            if can_move_to_world(self.world, density, x, y - 1) {
                self.world.move_cell_diff_chunk(x, y, x, y - 1, chunk);
                return true;
            }
        }
        false
    }

    fn move_up(&self, x: i32, y: i32, density: f32, chunk: &mut PixelChunk) -> bool {
        if self.world.inside_chunk(chunk, (x, y + 1)) {
            if can_move_to(chunk, density, x, y + 1) {
                self.move_cell_same_chunk(x, y, x, y + 1, chunk);
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
                (can_move_to(chunk, density, x - 1, y - 1), true)
            } else {
                (can_move_to_world(self.world, density, x - 1, y - 1), false)
            }
        };
        let (mut down_right, down_right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y - 1)) {
                (can_move_to(chunk, density, x + 1, y - 1), true)
            } else {
                (can_move_to_world(self.world, density, x + 1, y - 1), false)
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
                (can_move_to(chunk, density, x - 1, y + 1), true)
            } else {
                (can_move_to_world(self.world, density, x - 1, y + 1), false)
            }
        };
        let (mut up_right, up_right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y + 1)) {
                (can_move_to(chunk, density, x + 1, y + 1), true)
            } else {
                (can_move_to_world(self.world, density, x + 1, y + 1), false)
            }
        };
        if up_left && up_right {
            up_left = rand::thread_rng().gen_bool(0.5);
            up_right = !up_left;
        }

        if up_left && up_left_inside {
            self.move_cell_same_chunk(x, y, x - 1, y + 1, chunk);
        }
        else if up_right && up_right_inside {
            self.move_cell_same_chunk(x, y, x + 1, y + 1, chunk);
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
                (can_move_to(chunk, density, x - 1, y), true)
            } else {
                (can_move_to_world(self.world, density, x - 1, y), false)
            }
        };
        let (mut right, right_inside) = {
            if self.world.inside_chunk(chunk, (x + 1, y)) {
                (can_move_to(chunk, density, x + 1, y), true)
            } else {
                (can_move_to_world(self.world, density, x + 1, y), false)
            }
        };
        if left && right {
            left = rand::thread_rng().gen_bool(0.5);
            right = !left;
        }

        if left && left_inside {
            self.move_cell_same_chunk(x, y, x - 1, y, chunk);
        }
        else if right && right_inside {
            self.move_cell_same_chunk(x, y, x + 1, y, chunk);
        }
        else if left {
            self.world.move_cell_diff_chunk(x, y, x - 1, y, chunk);
        }
        else if right {
            self.world.move_cell_diff_chunk(x, y, x + 1, y, chunk);
        }

        left || right
    }

    pub fn move_cell_same_chunk(&self, x: i32, y: i32, xto: i32, yto: i32, chunk: &mut PixelChunk) {
        let from_idx = chunk.get_index(x, y);
        chunk.changes.push((None, from_idx, chunk.get_index(xto, yto)));
    }
}

#[inline]
fn can_move_to(chunk: &PixelChunk, density_from: f32, xto: i32, yto: i32) -> bool {
    if chunk.in_bounds_world(xto, yto) {
        let cell = chunk.cells[chunk.get_index(xto, yto)];
        return cell.get_type() == CellType::Empty || should_move_density(density_from, cell.get_density());
    }
    false
}

#[inline]
fn can_move_to_world(world: &PixelWorld, density_from: f32, xto: i32, yto: i32) -> bool {
    match world.get_cell(xto, yto) {
        Some(cell) => cell.get_type() == CellType::Empty || should_move_density(density_from, cell.get_density()),
        None => false
    }
}

#[inline]
fn world_to_chunk_coords(chunk: &PixelChunk, x: i32, y: i32) -> (i32, i32) {
    (x % chunk.width, y % chunk.height)
}