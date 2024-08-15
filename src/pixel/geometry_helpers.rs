use bevy::math::{IVec2, Vec2};

pub const VEC_UP: IVec2 = IVec2 { x: 0, y: 1 };
pub const VEC_UP_LEFT: IVec2 = IVec2 { x: -1, y: 1 };
pub const VEC_UP_RIGHT: IVec2 = IVec2 { x: 1, y: 1 };
pub const VEC_DOWN: IVec2 = IVec2 { x: 0, y: -1 };
pub const VEC_DOWN_LEFT: IVec2 = IVec2 { x: -1, y: -1 };
pub const VEC_DOWN_RIGHT: IVec2 = IVec2 { x: 1, y: -1 };
pub const VEC_RIGHT: IVec2 = IVec2 { x: 1, y: 0 };
pub const VEC_LEFT: IVec2 = IVec2 { x: -1, y: 0 };

pub const DIRECTIONS: [IVec2; 9] = [
    VEC_DOWN_LEFT,
    VEC_DOWN,
    VEC_DOWN_RIGHT,
    VEC_LEFT,
    IVec2::ZERO,
    VEC_RIGHT,
    VEC_UP_LEFT,
    VEC_UP,
    VEC_UP_RIGHT,
];

// Like IRect but can be a line
#[derive(Clone, Copy, Debug)]
pub struct BoundRect {
    pub min: IVec2,
    pub max: IVec2,
}

impl BoundRect {

    pub fn empty() -> Self {
        BoundRect {
            min: IVec2::MAX,
            max: IVec2::MIN,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min == IVec2::MAX && self.max == IVec2::MIN || self.min.x > self.max.x || self.min.y > self.max.y
    }

    pub fn from_points(points: &[IVec2]) -> Self {
        if points.len() == 0 {
            return Self::empty()
        }

        // Find min and max
        let mut min = IVec2::MAX;
        let mut max = IVec2::MIN;
        for point in points.iter() {
            min = IVec2::min(min, *point);
            max = IVec2::max(max, *point);
        }
        Self {
            min,
            max
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        if !self.is_empty() && other.is_empty() {
            return *self;
        } else if self.is_empty() && !other.is_empty() {
            return *other;
        } else if self.is_empty() && other.is_empty() {
            return Self::empty();
        }

        let min = IVec2::min(self.min, other.min);
        let max = IVec2::max(self.max, other.max);

        Self {
            min,
            max
        }
    }

    pub fn union_point(&self, point: &IVec2) -> Self {
        let mut new_bound = *self;

        if self.is_empty() {
            return Self {
                min: *point,
                max: *point,
            };
        } else {
            new_bound.min = IVec2::min(self.min, *point);
            new_bound.max = IVec2::max(self.max, *point);

            return new_bound;
        }
    }

    pub fn contains(&self, point: &IVec2) -> bool {
        point.x >= self.min.x && point.y >= self.min.y && point.x <= self.max.x && point.y <= self.max.y
    }

    pub fn center(&self) -> IVec2 {
        IVec2::new(
            (self.min.x + self.max.x) / 2,
            (self.min.y + self.max.y) / 2
        )
    }

    // We need to offset when displaying the rect
    pub fn center_display(&self) -> Vec2 {
        let center = self.center() + IVec2::ONE;
        center.as_vec2()
    }

    pub fn size(&self) -> IVec2 {
        IVec2::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y
        )
    }
}