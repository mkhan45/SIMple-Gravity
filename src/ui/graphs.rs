use bevy_ecs::prelude::*;

use crate::physics::KinematicBody;

pub struct Graph {
    pub points: Vec<f32>,
    pub label: String,
    pub point_fn: rhai::FnPtr,
}

pub fn graph_points_sys(_graph_query: Query<&Graph>, _body_query: Query<&KinematicBody>) {
    todo!()
}
