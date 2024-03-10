use bevy::math::Vec2;

use crate::cell_types::{CellType, DirectionType, StateType};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    cell_color: [u8; 4],
    cell_movement: DirectionType, // Direction of cell movement (can have multiple)
    cell_type: StateType, // Type of cell
    velocity: Vec2,
}

impl Cell {
    pub fn new(ctype: CellType, dtype: DirectionType) -> Self {

        let cell_color = ctype.cell_color();

        Self {
            cell_type: ctype.into(),
            cell_color,
            cell_movement: dtype,
            velocity: Vec2::new(0.0, 0.0),
        }
    }

    // Constant empty cell (because empty cell is common)
    pub fn empty() -> Self {
        Self {
            cell_type: CellType::Empty.into(),
            cell_color: CellType::Empty.cell_color(),
            cell_movement: DirectionType::NONE,
            velocity: Vec2::new(0.0, 0.0),
        }
    }

    pub fn get_movement(&self) -> DirectionType {
        self.cell_movement
    }

    pub fn get_state_type(&self) -> StateType {
        self.cell_type
    }

    pub fn get_type(&self) -> CellType {
        match self.cell_type {
            StateType::Empty(ctype) => ctype,
            StateType::SoftSolid(ctype) => ctype,
            StateType::HardSolid(ctype) => ctype,
            StateType::Liquid(ctype) => ctype,
            StateType::Gas(ctype) => ctype,
        }
    }

    pub fn get_color(&self) -> &[u8; 4] {
        &self.cell_color
    }

    pub fn get_density(&self) -> f32 {
        self.get_type().cell_density()
    }
}

impl From<CellType> for Cell {
    fn from(ctype: CellType) -> Self {
        match ctype {
            CellType::Empty => Self::empty(),
            CellType::Sand => Self::new(CellType::Sand,
                 DirectionType::DOWN | DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT),
            CellType::Dirt => Self::new(CellType::Dirt,
                 DirectionType::DOWN | DirectionType::DOWN_LEFT | DirectionType::DOWN_RIGHT),
            CellType::Stone => Self::new(CellType::Stone,
                 DirectionType::NONE),
            CellType::Water => Self::new(CellType::Water,
                 DirectionType::DOWN | DirectionType::LEFT | DirectionType::RIGHT),
        }
    }
}