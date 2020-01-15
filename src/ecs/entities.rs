use crate::ecs::components::*;
use crate::{Point, Vector};
use crate::ecs::components::SaveMarker;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};

use std::collections::VecDeque;

use ggez::graphics;

pub type Body = (Position, Kinematics, Mass, Draw, Radius, Trail);
pub type PreviewBody = (Position, Kinematics, Radius, Preview, Draw, Trail);

pub fn create_body(world: &mut World, body: Body) -> Entity {
    world
        .create_entity()
        .with(body.0)
        .with(body.1)
        .with(body.2)
        .with(body.3)
        .with(body.4)
        .with(body.5)
        .marked::<SimpleMarker<SaveMarker>>()
        .build()
}

pub fn create_preview(world: &mut World, body: PreviewBody) -> Entity {
    world
        .create_entity()
        .with(body.0)
        .with(body.1)
        .with(body.2)
        .with(body.3)
        .with(body.4)
        .with(body.5)
        .build()
}

pub fn new_body(pos: impl Into<Point>, vel: impl Into<Vector>, mass: f32, rad: f32) -> Body {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Mass(mass),
        Draw(ggez::graphics::WHITE),
        Radius(rad),
        Trail(VecDeque::with_capacity(36)),
    )
}

pub fn new_preview(pos: impl Into<Point>, vel: impl Into<Vector>, rad: f32) -> PreviewBody {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Radius(rad),
        Preview,
        Draw(graphics::Color::new(0.1, 1.0, 0.2, 0.8)),
        Trail(VecDeque::with_capacity(500)),
    )
}
