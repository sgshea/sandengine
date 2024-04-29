use crate::{cell::Cell, cell_types::CellType};

#[derive(Debug, Clone)]
pub struct PixelChunk {
    pub width: i32,
    pub height: i32,

    pub pos_x: i32,
    pub pos_y: i32,

    pub cells: Vec<Cell>,

    pub awake: bool,
    pub awake_next: bool,
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
            awake: true,
            awake_next: true,
        };
        
        s
    }

    pub fn get_pos(&self) -> (i32, i32) {
        (self.pos_x, self.pos_y)
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

    pub fn in_bounds_world(&self, x: i32, y: i32) -> bool {
        let idx = self.get_index(x, y);
        idx < self.cells.len()
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        let idx = self.get_index(x, y);
        idx < self.cells.len() && self.cells[idx].get_type() == CellType::Empty
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

    pub fn get_cell_mut(&mut self, idx: usize) -> &mut Cell {
        &mut self.cells[idx]
    }

    pub fn get_cell_mut_2d(&mut self, x: i32, y: i32) -> &mut Cell {
        let idx = self.get_index(x, y);
        if idx < self.cells.len() {
            &mut self.cells[idx]
        } else {
            println!("Index out of bounds: {} {} {}", x, y, idx);
            panic!("Chunk: {} {} {} {}", self.pos_x, self.pos_y, self.width, self.height);
        }
    }

    pub fn set_cell_1d(&mut self, idx: usize, cell: Cell) {
        if idx < self.cells.len() {
            self.cells[idx] = cell;
            self.awake_next = true;
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

    pub fn split_top_bot(&mut self) -> (Vec<&mut Cell>, Vec<&mut Cell>) {
        let mid = self.cells.len() / 2;
        let (top, bottom) = self.cells.split_at_mut(mid);
        (top.iter_mut().collect(), bottom.iter_mut().collect())
    }

    pub fn get_side_cells(&mut self) -> (Vec<&mut Cell>, Vec<&mut Cell>) {
        let side_length = (self.cells.len() as f64).sqrt() as usize;
        let half = side_length / 2;
        let mut cells_l = Vec::new();
        let mut cells_r = Vec::new();
    
        for i in 0..side_length {
            let start = i * side_length;
            let ptr = self.cells.as_mut_ptr();
    
            // UNSAFE
            // This is unsafe because it uses raw pointers and could possibly create invalid slices
            // This is safe because cells is guaranteed to exist, and all elements are initialized
            // The slice will be valid and always is within the bounds of the original data, and not overlapping with other slices
            unsafe {
                let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
                for cell in slice {
                    cells_l.push(cell);
                }
            }
        }
        for i in 0..side_length {
            let start = i * side_length + half;
            let ptr = self.cells.as_mut_ptr();
    
            // UNSAFE
            // This is unsafe because it uses raw pointers and could possibly create invalid slices
            // This is safe because cells is guaranteed to exist, and all elements are initialized
            // The slice will be valid and always is within the bounds of the original data, and not overlapping with other slices
            unsafe {
                let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
                for cell in slice {
                    cells_r.push(cell);
                }
            }
        }

        (cells_l, cells_r)
    }
}

pub enum SplitChunk<'a> {
    Entire(&'a mut PixelChunk),

    TopBottom([Option<Vec<&'a mut Cell>>; 2]),

    LeftRight([Option<Vec<&'a mut Cell>>; 2]),

    // Forgoing corners for now
    Corners([Option<Vec<&'a mut Cell>>; 4]),
}

impl SplitChunk<'_> {
    // create from a mutable reference to a PixelChunk
    pub fn from_chunk(chunk: &mut PixelChunk) -> SplitChunk {
        SplitChunk::Entire(chunk)
    }

    pub fn from_chunk_vert(chunk: &mut PixelChunk) -> SplitChunk {
        let (top, bottom) = chunk.split_top_bot();
        SplitChunk::TopBottom([Some(top), Some(bottom)])
    }

    pub fn from_chunk_side(chunk: &mut PixelChunk) -> SplitChunk {
        let (left, right) = chunk.get_side_cells();
        SplitChunk::LeftRight([Some(left), Some(right)])
    }
}