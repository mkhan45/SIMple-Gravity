use mint::{Point2, Vector2};

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kinematics {
    pub pos: Point,
    pub vel: Vector,
    pub accel: Vector,
}

impl Kinematics {
    pub fn new(pos: impl Into<Point>) -> Self {
        Kinematics {
            pos: pos.into(),
            vel: [0.0, 0.0].into(),
            accel: [0.0, 0.0].into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass(f32);
