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

    pub fn get_index(&self, x: i32, y: i32) -> usize {
        // world to chunk coord
        let x = x % self.width;
        let y = y % self.height;

        (y * self.width + x) as usize
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
            self.awake_next = true;
        }
    }

    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        let idx = self.get_index(x, y);
        self.set_cell_1d(idx, cell);
    }

    pub fn cells_as_floats(&self) -> Vec<f64> {
        // Map each cell to a float depending on if it is solid
        // range 0.0-1.0

        self.cells.iter().map(|cell| {
            if cell.get_type() == CellType::Empty {
                0.0
            } else {
                1.0
            }
        }).collect::<Vec<f64>>()
    }
}

fn split_top_bottom_cells(cells: &mut Vec<Cell>) -> (Vec<&mut Cell>, Vec<&mut Cell>) {
    let mid = cells.len() / 2;
    let (top, bottom) = cells.split_at_mut(mid);
    (top.iter_mut().collect(), bottom.iter_mut().collect())
}

fn split_left_right_cells(cells: &mut Vec<Cell>) -> (Vec<&mut Cell>, Vec<&mut Cell>) {
    let side_length = (cells.len() as f64).sqrt() as usize;
    let half = side_length / 2;
    let mut cells_l = Vec::new();
    let mut cells_r = Vec::new();

    for i in 0..side_length {
        let start = i * side_length;
        let ptr = cells.as_mut_ptr();

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
        let ptr = cells.as_mut_ptr();

        // UNSAFE
        // This is unsafe because it uses raw pointers and could possibly create invalid slices
        // This is safe because cells is guaranteed to exist, and all elements are initialized
        // The slice will be valid and always is within the bounds of the original data, and not overlapping with other slices
        // Playground demonstration: https://gist.github.com/rust-play/00f1c05433719be6dc3add0b8c10df14
        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
            for cell in slice {
                cells_r.push(cell);
            }
        }
    }

    (cells_l, cells_r)
}

fn split_corner_cells(cells: &mut Vec<Cell>) -> (Vec<&mut Cell>, Vec<&mut Cell>, Vec<&mut Cell>, Vec<&mut Cell>) {
    let side_length = (cells.len() as f64).sqrt() as usize;
    // Get top and bottom
    let mid = cells.len() / 2;
    let (top, bottom) = cells.split_at_mut(mid);

    // Get left and right from top
    let half = side_length / 2;
    let mut cells_tl = Vec::new();
    let mut cells_tr = Vec::new();

    for i in 0..half {
        let start = i * side_length;
        let ptr = top.as_mut_ptr();

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
            for cell in slice {
                cells_tl.push(cell);
            }
        }
    }
    for i in 0..half {
        let start = i * side_length + half;
        let ptr = top.as_mut_ptr();

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
            for cell in slice {
                cells_tr.push(cell);
            }
        }
    }

    // Get left and right from bottom
    let mut cells_bl = Vec::new();
    let mut cells_br = Vec::new();

    for i in 0..half {
        let start = i * side_length;
        let ptr = bottom.as_mut_ptr();

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
            for cell in slice {
                cells_bl.push(cell);
            }
        }
    }
    for i in 0..half {
        let start = i * side_length + half;
        let ptr = bottom.as_mut_ptr();

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.add(start), half);
            for cell in slice {
                cells_br.push(cell);
            }
        }
    }

    (cells_tl, cells_tr, cells_bl, cells_br)
}

pub enum SplitChunk<'a> {
    Entire(&'a mut PixelChunk),

    TopBottom([Option<Vec<&'a mut Cell>>; 2]),

    LeftRight([Option<Vec<&'a mut Cell>>; 2]),

    Corners([Option<Vec<&'a mut Cell>>; 4]),
}

impl SplitChunk<'_> {
    // Borrowing cells from the chunk
    pub fn from_chunk(chunk: &mut PixelChunk) -> SplitChunk {
        SplitChunk::Entire(chunk)
    }

    pub fn from_chunk_vert(chunk: &mut PixelChunk) -> SplitChunk {
        let (top, bottom) = split_top_bottom_cells(&mut chunk.cells);
        SplitChunk::TopBottom([Some(top), Some(bottom)])
    }

    pub fn from_chunk_side(chunk: &mut PixelChunk) -> SplitChunk {
        let (left, right) = split_left_right_cells(&mut chunk.cells);
        SplitChunk::LeftRight([Some(left), Some(right)])
    }

    pub fn from_chunk_corners(chunk: &mut PixelChunk) -> SplitChunk {
        let (tl, tr, bl, br) = split_corner_cells(&mut chunk.cells);
        SplitChunk::Corners([Some(tl), Some(tr), Some(bl), Some(br)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_top_bottom_cells() {
        let mut chunk = PixelChunk::new(16, 16, 0, 0);

        let (top, bottom) = split_top_bottom_cells(&mut chunk.cells);
        assert_eq!(top.len(), 128);
        assert_eq!(bottom.len(), 128);
    }

    #[test]
    fn test_split_left_right_cells() {
        let mut chunk = PixelChunk::new(16, 16, 0, 0);

        let (left, right) = split_left_right_cells(&mut chunk.cells);
        assert_eq!(left.len(), 128);
        assert_eq!(right.len(), 128);
    }

    #[test]
    fn test_split_corner_cells() {
        let mut chunk = PixelChunk::new(16, 16, 0, 0);

        let (top_left, top_right, bottom_left, bottom_right) = split_corner_cells(&mut chunk.cells);
        // each should be 8x8

        assert_eq!(top_left.len(), 64);
        assert_eq!(top_right.len(), 64);
        assert_eq!(bottom_left.len(), 64);
        assert_eq!(bottom_right.len(), 64);
    }
}