use ggez::graphics::Color;
use ggez::nalgebra::{Point2, Vector2};

use std::collections::VecDeque;

use specs::prelude::*;
use specs::Component;

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;

#[derive(Default, Clone, Copy, Debug, PartialEq, Component)]
#[storage(NullStorage)]
pub struct Preview;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

impl Position {
    pub fn dist(self, p2: impl Into<Point>) -> f32 {
        (self.0 - p2.into()).norm()
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

impl From<[f32; 2]> for Position {
    fn from(p: [f32; 2]) -> Self {
        Position(p.into())
    }
}

impl From<Point> for Position {
    fn from(p: Point) -> Self {
        Position(p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
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

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Mass(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Draw(pub Color);

#[derive(Default, Clone, Copy, Debug, PartialEq, Component)]
#[storage(NullStorage)]
pub struct Static;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Radius(pub f32);

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Trail(pub VecDeque<Point>);

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(HashMapStorage)]
pub struct SpeedGraph {
    pub data: Vec<f32>,
    pub display: bool,
}

impl SpeedGraph {
    pub fn new() -> Self {
        SpeedGraph {
            data: Vec::with_capacity(500),
            display: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(HashMapStorage)]
pub struct XVelGraph {
    pub data: Vec<f32>,
    pub display: bool,
}

impl XVelGraph {
    pub fn new() -> Self {
        XVelGraph {
            data: Vec::with_capacity(500),
            display: true,
        }
    }
}
