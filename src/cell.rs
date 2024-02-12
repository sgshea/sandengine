use crate::cell_types::{CellType, DirectionType};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    cell_color: [f32; 3],
    cell_movement: DirectionType, // Direction of cell movement (can have multiple)
    cell_type: CellType, // Type of cell
}

impl Cell {
    pub fn new(ctype: CellType, dtype: DirectionType) -> Self {
        // godot_print!("Cell mem size {}", std::mem::size_of::<Self>());

        // Cell colors as u32
        let cell_color = match ctype {
            CellType::Empty => [0.0, 0.0, 0.0],
            CellType::Sand => [0.85, 0.80, 0.5],
            CellType::Stone => [0.49, 0.43, 0.43],
            CellType::Water => [0.48, 0.6, 0.78],
        };

        Self {
            cell_type: ctype,
            cell_color,
            cell_movement: dtype,
        }
    }

    // Constant empty cell (because empty cell is common)
    pub fn empty() -> Self {
        Self {
            cell_type: CellType::Empty,
            cell_color: [0.0, 0.0, 0.0],
            cell_movement: DirectionType::NONE,
        }
    }

    // Construct a cell using the type (this is what should be used most of the time)
    pub fn cell_from_type(ctype: CellType) -> Self {
        match ctype {
            CellType::Empty => Self::empty(),
            CellType::Sand => Self::new(CellType::Sand,
                 DirectionType::DOWN | DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT),
            CellType::Stone => Self::new(CellType::Stone,
                 DirectionType::NONE),
            CellType::Water => Self::new(CellType::Water,
                 DirectionType::DOWN | DirectionType::LEFT | DirectionType::RIGHT),
        }
    }


    pub fn get_cell_movement(&self) -> DirectionType {
        self.cell_movement
    }

    pub fn get_cell_type(&self) -> CellType {
        self.cell_type.clone()
    }

    pub fn get_cell_color(&self) -> &[f32; 3] {
        &self.cell_color
    }

    pub fn set_cell_type(&mut self, ctype: CellType) {
        self.cell_type = ctype;
    }
}