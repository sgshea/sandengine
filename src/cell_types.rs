use bitflags::bitflags;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CellType {
    Empty = 0,
    Sand = 1,
    Stone = 2,
    Water = 3,
}

// Not used yet, will probably be used once CellType starts expanding
#[derive(Clone, Copy, Debug)]
pub enum StateType {
    Empty,
    SoftSolid, // Soft solid, like sand that can move
    HardSolid, // Hard solid, like stone that can't move
    Liquid,
    Gas,
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