use rand::Rng;

use crate::{cell::Cell, cell_types::{CellType, DirectionType}, chunk::PixelChunk, world::PixelWorld, cworker::*};

// CellEffect strategy pattern
// Interface to allow some effect such as movement or fire spread to be applied from a cell to surrounding cells
// The interface is provided the cell to apply the effect to and a list of surrounding cells
pub trait CellEffect {
    fn apply(&self, cell: Cell, cell_location: (i32, i32), other_cells: Vec<(DirectionType, Cell)>, worker: &ChunkWorker, original_chunk: &mut PixelChunk);
}

fn check_cell_exists(dir: DirectionType, other_cells: &Vec<(DirectionType, Cell)>) -> bool {
    for (direction, _) in other_cells {
        if dir == *direction {
            return true;
        }
    }
    false
}

// Movement effect
// Moves a cell based on it's movement direction
pub struct MovementEffect;
impl CellEffect for MovementEffect {
    fn apply(&self, cell: Cell, cell_location: (i32, i32), other_cells: Vec<(DirectionType, Cell)>, worker: &ChunkWorker, original_chunk: &mut PixelChunk) {
        // Get the movement direction of the cell
        let movement = cell.get_movement();
        if movement.intersects(DirectionType::DOWN) && check_cell_exists(DirectionType::DOWN, &other_cells) {
            // try move down
            if can_move_to(worker.world, &original_chunk, cell_location.0, cell_location.1 - 1) {
                worker.move_cell(cell_location, DirectionType::DOWN, original_chunk);
                return;
            }
        }
        if movement.intersects(DirectionType::UP) && check_cell_exists(DirectionType::UP, &other_cells) {
            // try move up
            if can_move_to(worker.world, &original_chunk, cell_location.0, cell_location.1 + 1) {
                worker.move_cell(cell_location, DirectionType::UP, original_chunk);
                return;
            }
        }
        if movement.intersects(DirectionType::LEFT | DirectionType::RIGHT) && (check_cell_exists(DirectionType::LEFT, &other_cells) || check_cell_exists(DirectionType::RIGHT, &other_cells)) {
            // try move left or right
            // check both locations and random chance move
            let can_move_left = can_move_to(worker.world, &original_chunk, cell_location.0 - 1, cell_location.1);
            let can_move_right = can_move_to(worker.world, &original_chunk, cell_location.0 + 1, cell_location.1);
            if can_move_left && can_move_right {
                let mut trng = rand::thread_rng();
                let random_number: f32 = trng.gen_range(0.0..1.0);
                if random_number < 0.5 {
                    worker.move_cell(cell_location, DirectionType::LEFT, original_chunk);
                    return;
                } else {
                    worker.move_cell(cell_location, DirectionType::RIGHT, original_chunk);
                    return;
                }
            } else if can_move_left {
                worker.move_cell(cell_location, DirectionType::LEFT, original_chunk);
                return;
            } else if can_move_right {
                worker.move_cell(cell_location, DirectionType::RIGHT, original_chunk);
                return;
            }
        }
        if movement.intersects(DirectionType::UP_LEFT | DirectionType::UP_RIGHT) && (check_cell_exists(DirectionType::UP_LEFT, &other_cells) || check_cell_exists(DirectionType::UP_RIGHT, &other_cells)) {
            // try move up left or up right
            // check both locations and random chance move
            let can_move_up_left = can_move_to(worker.world, &original_chunk, cell_location.0 - 1, cell_location.1 + 1);
            let can_move_up_right = can_move_to(worker.world, &original_chunk, cell_location.0 + 1, cell_location.1 + 1);
            if can_move_up_left && can_move_up_right {
                let mut trng = rand::thread_rng();
                let random_number: f32 = trng.gen_range(0.0..1.0);
                if random_number < 0.5 {
                    worker.move_cell(cell_location, DirectionType::UP_LEFT, original_chunk);
                    return;
                } else {
                    worker.move_cell(cell_location, DirectionType::UP_RIGHT, original_chunk);
                    return;
                }
            } else if can_move_up_left {
                worker.move_cell(cell_location, DirectionType::UP_LEFT, original_chunk);
                return;
            } else if can_move_up_right {
                worker.move_cell(cell_location, DirectionType::UP_RIGHT, original_chunk);
                return;
            }
        }
        if movement.intersects(DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT) && (check_cell_exists(DirectionType::DOWN_LEFT, &other_cells) || check_cell_exists(DirectionType::DOWN_RIGHT, &other_cells)) {
            // try move down left or down right
            // check both locations and random chance move
            let can_move_down_left = can_move_to(worker.world, &original_chunk, cell_location.0 - 1, cell_location.1 - 1);
            let can_move_down_right = can_move_to(worker.world, &original_chunk, cell_location.0 + 1, cell_location.1 - 1);
            if can_move_down_left && can_move_down_right {
                let mut trng = rand::thread_rng();
                let random_number: f32 = trng.gen_range(0.0..1.0);
                if random_number < 0.5 {
                    worker.move_cell(cell_location, DirectionType::DOWN_LEFT, original_chunk);
                    return;
                } else {
                    worker.move_cell(cell_location, DirectionType::DOWN_RIGHT, original_chunk);
                    return;
                }
            } else if can_move_down_left {
                worker.move_cell(cell_location, DirectionType::DOWN_LEFT, original_chunk);
                return;
            } else if can_move_down_right {
                worker.move_cell(cell_location, DirectionType::DOWN_RIGHT, original_chunk);
                return;
            }
        }
    }
}

#[inline]
fn can_move_to(world: &PixelWorld, chunk: &PixelChunk, xto: i32, yto: i32) -> bool {
    if chunk.in_bounds_world(xto, yto) {
        let cell = chunk.cells[chunk.get_index(xto, yto)];
        cell.get_type() == CellType::Empty
    } else if world.in_bounds(xto, yto) {
        match world.get_cell(xto, yto) {
            Some(cell) => cell.get_type() == CellType::Empty,
            None => false
        }
    } else {
        false
    }
}