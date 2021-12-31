use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::{
    physics::{KinematicBody, Preview},
    trails::Trail,
};

pub struct PreviewTrailTick {
    pub current_tick: u8,
    pub tick_increment: u8,
}

pub struct MultiPreview(pub bool);

impl Default for PreviewTrailTick {
    fn default() -> Self {
        Self {
            current_tick: 0,
            tick_increment: 10,
        }
    }
}

pub fn preview_gravity_sys(
    query_set: QuerySet<(
        Query<(&mut KinematicBody, &Trail), With<Preview>>,
        Query<&KinematicBody, Without<Preview>>,
    )>,
) {
    let preview_query = query_set.q0();
    let body_query = query_set.q1();

    unsafe {
        for (mut preview_body, trail) in preview_query.iter_unsafe() {
            if trail.points.len() == trail.max_len {
                preview_body.vel = Vec2::new(0.0, 0.0);
                continue;
            }

            let mut cumulative_force = Vec2::new(0.0, 0.0);
            let p1 = preview_body.pos;
            let m1 = preview_body.mass;

            for affecting_body in body_query.iter() {
                let p2 = affecting_body.pos;
                let m2 = affecting_body.mass;

                let rad = p2 - p1;
                let rad_sqr_dist = rad.length_squared();
                let rad_dist = rad_sqr_dist.powf(0.5);

                let current_force = crate::physics::G * m1 * m2 / rad_sqr_dist;

                cumulative_force.x += current_force * rad.x / rad_dist;
                cumulative_force.y += current_force * rad.y / rad_dist;
            }

            preview_body.force += cumulative_force;
        }
    }
}
