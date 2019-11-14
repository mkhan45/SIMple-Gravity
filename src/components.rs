use ggez::graphics::Color;
use nalgebra::{Point2, Vector2};

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Point);
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw(pub Color);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Static;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius(pub f32);
