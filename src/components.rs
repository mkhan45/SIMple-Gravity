use nalgebra::{Point2, Vector2};
use ggez::graphics::Color;

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Point);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub Vector);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Acceleration(pub Vector);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw(pub Color);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Static;
