use bevy_egui::egui::epaint::color;
use rand::Rng;
use strum::{EnumIter, VariantNames};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cell {
    pub color: [u8; 4],

    pub physics: PhysicsType,

    pub updated: bool,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, EnumIter, VariantNames, Default)]
pub(crate) enum CellType {
    #[default]
    Empty,
    Sand,
    Dirt,
    Stone,
    Water,
    Smoke,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, Default)]
pub(crate) enum PhysicsType {
    #[default]
    Empty,
    // Soft solid, like sand that can move
    SoftSolid(CellType),
    // Hard solid, like stone that can't move
    HardSolid(CellType),
    // Liquid type such as water
    Liquid(CellType),
    // Gas type such as as smoke
    Gas(CellType),
    // Special case for rigid bodies which don't use cell physics but still contain cells
    RigidBody(CellType),
}

impl CellType {

    pub fn cell_color(&self) -> [u8; 4] {
        let mut trng = rand::thread_rng();
        match self {
            CellType::Empty => [0, 0, 0, 0],
            CellType::Sand => {
                [
                    (230 + trng.gen_range(-20..20)) as u8,
                    (195 + trng.gen_range(-20..20)) as u8,
                    (92 + trng.gen_range(-20..20)) as u8,
                    255,
                ]
            },
            CellType::Dirt => {
                [
                    (139 + trng.gen_range(-10..10)) as u8,
                    (69 + trng.gen_range(-10..10)) as u8,
                    (19 + trng.gen_range(-10..10)) as u8,
                    255,
                ]
            },
            CellType::Stone => {
                [
                    (80 + trng.gen_range(-10..10)) as u8,
                    (80 + trng.gen_range(-10..10)) as u8,
                    (80 + trng.gen_range(-10..10)) as u8,
                    255,
                ]
            },
            CellType::Water => {
                [
                    (20 + trng.gen_range(-20..20)) as u8,
                    (125 + trng.gen_range(-20..20)) as u8,
                    (205 + trng.gen_range(-20..20)) as u8,
                    150,
                ]
            },
            CellType::Smoke => {
                [
                    (192 + trng.gen_range(-20..20)) as u8,
                    (192 + trng.gen_range(-20..20)) as u8,
                    (192 + trng.gen_range(-20..20)) as u8,
                    150,
                ]
            },
        }
    }
}

impl From<CellType> for PhysicsType {
    fn from(ctype: CellType) -> Self {
        match ctype {
            CellType::Empty => PhysicsType::Empty,
            CellType::Sand => PhysicsType::SoftSolid(ctype),
            CellType::Dirt => PhysicsType::SoftSolid(ctype),
            CellType::Stone => PhysicsType::HardSolid(ctype),
            CellType::Water => PhysicsType::Liquid(ctype),
            CellType::Smoke => PhysicsType::Gas(ctype),
        }
    }
}

impl Cell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            color: cell_type.cell_color(),
            physics: PhysicsType::from(cell_type),
            updated: false,
        }
    }

    pub fn with_cell_and_color(cell_type: CellType, color: [u8; 4]) -> Self {
        Self {
            color,
            physics: PhysicsType::from(cell_type),
            updated: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.physics == PhysicsType::Empty
    }
}

impl From<CellType> for Cell {
    fn from(value: CellType) -> Self {
        Cell::new(value)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            color: CellType::Empty.cell_color(),
            physics: PhysicsType::Empty,
            updated: false,
        }
    }
}