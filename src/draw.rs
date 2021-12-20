use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::KinematicBody;

pub fn draw_bodies_sys(query: Query<&KinematicBody>) {
    for body in query.iter() {
        draw_circle(body.pos.x, body.pos.y, body.radius, WHITE);
    }
}
