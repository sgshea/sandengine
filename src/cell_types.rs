use bitflags::bitflags;
use rand::Rng;
use strum::{EnumIter, VariantNames};

// Maximum density of a cell
const MAX_DENSITY: f32 = 100.0;

#[derive(Clone, Copy, Eq, PartialEq, Debug, EnumIter, VariantNames)]
pub enum CellType {
    Empty,
    Sand,
    Dirt,
    Stone,
    Water,
    Smoke,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Empty
    }
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

    // Inertia is how likely a cell will choose to stay in place (not look sideways for a new cell to move to)
    pub fn cell_inertia(&self) -> f32 {
        match self {
            CellType::Empty => 0.0,
            CellType::Sand => 0.5,
            CellType::Dirt => 0.65,
            CellType::Stone => 0.9,
            CellType::Water => 0.4,
            CellType::Smoke => 0.2,
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

// What kind of cell state is it?
// Used to determine simple behaviors, but allows access to a more specific CellType
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum StateType {
    Empty(CellType),
    SoftSolid(CellType), // Soft solid, like sand that can move
    HardSolid(CellType), // Hard solid, like stone that can't move
    Liquid(CellType),
    Gas(CellType),
}

impl Default for StateType {
    fn default() -> Self {
        StateType::Empty(CellType::Empty)
    }
}

impl From<CellType> for StateType {
    fn from(ctype: CellType) -> Self {
        match ctype {
            CellType::Empty => StateType::Empty(ctype),
            CellType::Sand => StateType::SoftSolid(ctype),
            CellType::Dirt => StateType::SoftSolid(ctype),
            CellType::Stone => StateType::HardSolid(ctype),
            CellType::Water => StateType::Liquid(ctype),
            CellType::Smoke => StateType::Gas(ctype),
        }
    }
}

// Direction stored as bitflags
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct DirectionType: u32 {
        const NONE = 0;
        const DOWN = 0b00000001;
        const DOWN_LEFT = 0b00000010;
        const DOWN_RIGHT = 0b00000100;
        const LEFT = 0b00001000;
        const RIGHT = 0b00010000;
        const UP = 0b00100000;
        const UP_LEFT = 0b01000000;
        const UP_RIGHT = 0b10000000;
    }
}

impl DirectionType {
    pub fn get_tuple_direction(self) -> (i32, i32) {
        match self {
            DirectionType::NONE => (0, 0),
            DirectionType::DOWN => (0, -1),
            DirectionType::DOWN_LEFT => (-1, -1),
            DirectionType::DOWN_RIGHT => (1, -1),
            DirectionType::LEFT => (-1, 0),
            DirectionType::RIGHT => (1, 0),
            DirectionType::UP => (0, 1),
            DirectionType::UP_LEFT => (-1, 1),
            DirectionType::UP_RIGHT => (1, 1),
            _ => (0, 0),
        }
    }
}