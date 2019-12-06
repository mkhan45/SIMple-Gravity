use crate::{Point, Vector};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewIterations(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainIterations(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartPoint(pub Option<Point>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution(pub Vector);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MousePos(pub Point);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DT(pub f32);

