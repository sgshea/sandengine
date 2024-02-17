use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::{cell::Cell, cell_types::CellType};

#[derive(Debug, Clone)]
pub struct PixelChunk {
    pub width: i32,
    pub height: i32,

    pub pos_x: i32,
    pub pos_y: i32,

    pub cells: Vec<Cell>,

    pub changes: Vec<(Option<Arc<Mutex<PixelChunk>>>, usize, usize)>,
}

impl PixelChunk {
    pub fn new(width: i32, height: i32, pos_x: i32, pos_y: i32) -> Self {
        let cells = vec![Cell::empty(); (height * width) as usize];

        let s = PixelChunk {
            width,
            height,
            pos_x,
            pos_y,
            cells,
            changes: Vec::new(),
        };
        
        s
    }

    pub fn get_index(&self, x: i32, y: i32) -> usize {
        // world to chunk coord
        let x = x % self.width;
        let y = y % self.height;

        (y * self.width + x) as usize
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        let idx = self.get_index(x, y);
        idx < self.cells.len() && self.cells[idx].get_cell_type() == CellType::Empty
    }

    pub fn get_cell(&self, idx: usize) -> &Cell {
        &self.cells[idx]
    }

    pub fn get_cell_2d(&self, x: i32, y: i32) -> &Cell {
        let idx = self.get_index(x, y);
        if idx < self.cells.len() {
            &self.cells[idx]
        } else {
            println!("Index out of bounds: {} {} {}", x, y, idx);
            panic!("Chunk: {} {} {} {}", self.pos_x, self.pos_y, self.width, self.height);
        }
    }

    pub fn set_cell_1d(&mut self, idx: usize, cell: Cell) {
        if idx < self.cells.len() {
            self.cells[idx] = cell;
        }
    }

    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        let idx = self.get_index(x, y);
        self.set_cell_1d(idx, cell);
    }

    pub fn set_cell_checked(&mut self, x: i32, y: i32, cell: Cell) {
        if self.in_bounds(x, y) {
            self.set_cell(x, y, cell);
        }
    }

    pub fn commit_cells(&mut self) {
        // Remove moves that have their destinations filled
        self.changes.retain(|(_, _, dst)| self.cells[*dst as usize].get_cell_type() == CellType::Empty);

        // Sort by destination
        self.changes.sort_by(|a, b| a.2.cmp(&b.2));

        // Iterate over sorted moves and pick random source to move from each time
        let mut iprev = 0;
        if self.changes.len() == 0 {
            return;
        }
        for i in 0..self.changes.len() {
            if i == self.changes.len() - 1 || self.changes[i + 1].2 != self.changes[i].2 {
                let rand = iprev + rand::thread_rng().gen_range(0..=(i - iprev));

                let dst = self.changes[rand].2;
                let src = self.changes[rand].1;
                match &self.changes[rand].0 {
                    Some(chunk) => {
                        let mut chunk = chunk.lock().unwrap();
                        self.cells[dst as usize] = chunk.cells[src as usize].clone();
                        chunk.cells[src as usize] = Cell::empty();
                    },
                    None => {
                        self.cells[dst as usize] = self.cells[src as usize].clone();
                        self.cells[src as usize] = Cell::empty();
                    }
                }
                iprev = i + 1;
            }
        }
        self.changes.clear();
    }
}