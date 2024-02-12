use rand::Rng;

use crate::{cell::Cell, cell_types::{CellType, DirectionType}};

pub struct PixelWorld {
    c_height: i32,
    c_width: i32,

    changes: Vec<(i32, i32)>, // destination, source vector

    cells: Vec<Cell>,
}

impl PixelWorld {

    pub fn new(c_width: i32, c_height: i32) -> Self {
        let cells = vec![Cell::empty(); (c_height * c_width) as usize];

        let s = PixelWorld {
            c_width,
            c_height,
            changes: Vec::new(),
            cells,
        };
        
        s
    }

    pub fn get_all_cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn get_cell(&self, idx: i32) -> &Cell {
        &self.cells[idx as usize]
    }

    pub fn get_cell_2d(&self, x: i32, y: i32) -> &Cell {
        &self.cells[get_index(x, y, self.c_width) as usize]
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x < self.c_width && y < self.c_height && x >= 0 && y >= 0
    }

    fn is_empty(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.get_cell_2d(x, y).get_cell_type() == CellType::Empty
    }

    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        self.cells[get_index(x, y, self.c_width) as usize] = cell;
    }

    pub fn set_cell_checked(&mut self, x: i32, y: i32, cell: Cell) {
        if self.in_bounds(x, y) {
            self.set_cell(x, y, cell);
        }
    }

    fn move_cell(&mut self, x: i32, y: i32, xto: i32, yto: i32) {
        self.changes.push(
            (get_index(xto, yto, self.c_width), get_index(x, y, self.c_width))
        );
    }

    fn commit_cells(&mut self) {
        // Remove moves that have their destinations filled
        self.changes.retain(|(dst, _)| self.cells[*dst as usize].get_cell_type() == CellType::Empty);

        // Sort by destination
        self.changes.sort_by(|a, b| a.0.cmp(&b.0));

        // Iterate over sorted moves and pick random source to move from each time
        let mut iprev = 0;
        self.changes.push((-1, -1)); // catches final move
        for i in 0..self.changes.len() - 1 {
            if self.changes[i + 1].0 != self.changes[i].0 {
                let rand = iprev + rand::thread_rng().gen_range(0..=(i - iprev));

                let dst = self.changes[rand].0;
                let src = self.changes[rand].1;

                self.cells[dst as usize] = self.cells[src as usize].clone();
                self.cells[src as usize] = Cell::empty();
                iprev = i + 1;
            }
        }
        self.changes.clear();
    }

    // Update cells
    pub fn update(&mut self) {
        for x in 0..self.c_width {
            for y in 0..self.c_height {
                let cell_movement = self.get_cell_2d(x, y).get_cell_movement();


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

        self.commit_cells();
    }

    fn move_down(&mut self, x: i32, y: i32) -> bool {
        let down = self.is_empty(x, y - 1);
        if down {
            self.move_cell(x, y, x, y - 1);
        }
        down
    }

    fn move_diagonal(&mut self, x: i32, y: i32) -> bool {
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

    fn move_side(&mut self, x: i32, y: i32) -> bool {
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

// Calculates 1d index from 2d coordinates
pub fn get_index(x: i32, y: i32, width: i32) -> i32 {
    x + y * width
}