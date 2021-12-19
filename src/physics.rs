use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

// use std::sync::{Arc, Mutex};

pub struct DT(pub f32);
pub struct Steps(pub usize);

pub const G: f32 = 9.81;

#[derive(Default, Clone)]
pub struct KinematicBody {
    pub pos: Vec2,
    pub vel: Vec2,
    pub accel: Vec2,
    pub force: Vec2,
    pub mass: Vec2,
}

#[derive(Copy, Clone)]
pub struct CumulativeMass {
    pub pos: Vec2,
    pub mass: Vec2,
}

pub fn integrate_sys(mut query: Query<&mut KinematicBody>, dt: Res<DT>) {
    let dt = dt.0;

    for mut body in query.iter_mut() {
        let accel = body.accel;
        let vel = body.vel;
        body.pos += vel * dt + 0.5 * accel * dt * dt;

        let new_accel = body.accel + body.force * dt;
        body.vel = body.vel + (body.accel + new_accel) / 2.0;
        body.accel = new_accel;

        body.force = Vec2::new(0.0, 0.0);
    }
}

pub fn gravity_sys(affected_query: Query<&mut KinematicBody>, affecting_query: Query<&KinematicBody>) {
    unsafe {
        for mut affected_body in affected_query.iter_unsafe() {
            let mut cumulative_force = Vec2::default();
            let p1 = affected_body.pos;
            let m1 = affected_body.mass;

            for affecting_body in affecting_query.iter() {
                let p2 = affecting_body.pos;
                let m2 = affecting_body.mass;

                let rad = p2 - p1;
                let rad_sqr_dist = rad.length_squared();

                let current_force = G * m1 * m2 / rad_sqr_dist;

                cumulative_force += current_force;
            }

            // *affected_body.force.lock().unwrap() += cumulative_force;
            affected_body.force += cumulative_force;
        }
    }
}
