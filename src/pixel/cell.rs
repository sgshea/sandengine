use bevy::math::Vec2;
use rand::Rng;
use strum::{EnumIter, VariantNames};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cell {
    pub color: [u8; 4],
    pub velocity: Vec2,

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
        // Density is how likely a cell is to move through liquids
    pub fn cell_density(&self) -> f32 {
        match self {
            CellType::Empty => 0.0,
            CellType::Sand => 60.0,
            CellType::Dirt => 60.0,
            CellType::Stone => 100.0,
            CellType::Water => 50.0,
            CellType::Smoke => 10.0,
        }
    }

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

impl PhysicsType {
    pub fn density(&self) -> f32 {
        match self {
            PhysicsType::Empty => 0.0,
            PhysicsType::SoftSolid(cell) => cell.cell_density(),
            PhysicsType::HardSolid(cell) => cell.cell_density(),
            PhysicsType::Liquid(cell) => cell.cell_density(),
            PhysicsType::Gas(cell) => cell.cell_density(),
            PhysicsType::RigidBody(cell) => cell.cell_density(),
        }
    }
}

impl Cell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            color: cell_type.cell_color(),
            velocity: Vec2::ZERO,
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
            velocity: Vec2::ZERO,
            physics: PhysicsType::Empty,
            updated: false,
        }
    }
}