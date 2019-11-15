use ggez::graphics::Color;
use ggez::nalgebra::{Point2, Vector2};

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Point);

impl Position {
    pub fn dist(self, p2: Position) -> f32 {
        (self.0 - p2.0).norm()
    }
}

impl Into<Point> for Position {
    fn into(self) -> Point {
        self.0
    }
}

impl Into<ggez::mint::Point2<f32>> for Position {
    fn into(self) -> ggez::mint::Point2<f32> {
        [self.0.x, self.0.y].into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kinematics {
    pub vel: Vector,
    pub accel: Vector,
    pub past_accel: Vector,
}

impl Kinematics {
    pub fn new(vel: Vector) -> Self {
        Kinematics {
            vel,
            accel: Vector::new(0.0, 0.0),
            past_accel: Vector::new(0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw(pub Color);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Static;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius(pub f32);
