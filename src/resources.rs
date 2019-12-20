use crate::{Point, Vector};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PreviewIterations(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct MainIterations(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartPoint(pub Option<Point>);
impl Default for StartPoint {
    fn default() -> Self {
        StartPoint(Some(Point::new(0.0, 0.0)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution(pub Vector);
impl Default for Resolution {
    fn default() -> Self {
        Resolution(Vector::new(0.0, 0.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MousePos(pub Point);

impl Default for MousePos {
    fn default() -> Self {
        MousePos(Point::new(0.0, 0.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DT(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Paused(pub bool);

impl Paused {
    pub fn invert(&mut self) {
        self.0 = !self.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct NewPreview(pub bool);
