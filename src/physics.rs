use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

// use std::sync::{Arc, Mutex};

pub struct DT(pub f32);
pub struct Steps(pub usize);

pub const G: f32 = 100.0;

#[derive(Default, Clone)]
pub struct KinematicBody {
    pub pos: Vec2,
    pub vel: Vec2,
    pub accel: Vec2,
    pub force: Vec2,
    pub mass: f32,
    pub radius: f32,
}

#[derive(Copy, Clone)]
pub struct CumulativeMass {
    pub pos: Vec2,
    pub mass: Vec2,
}

pub struct Preview;

macro_rules! generate_integration_systems {
    ($filter:ident, $name:ident) => {
        pub fn $name(mut query: Query<&mut KinematicBody, $filter<Preview>>, dt: Res<DT>) {
            let dt = dt.0;

            for mut body in query.iter_mut() {
                let old_pos = body.pos;
                let old_vel = body.vel;
                let old_accel = body.accel;
                let new_accel = body.force / body.mass;

                let new_vel = old_vel + 0.5 * (new_accel + old_accel) * dt;
                let new_pos = old_pos + new_vel * dt + 0.5 * new_accel * dt * dt;

                body.vel = new_vel;
                body.pos = new_pos;
                body.accel = new_accel;

                body.force = Vec2::new(0.0, 0.0);
            }
        }
    };
}
generate_integration_systems!(Without, integration_sys);
generate_integration_systems!(With, preview_integration_sys);

pub fn gravity_sys(
    query_set: QuerySet<(
        Query<(&mut KinematicBody, Entity), Without<Preview>>,
        Query<(&KinematicBody, Entity), Without<Preview>>,
    )>,
) {
    let affected_query = query_set.q0();
    let affecting_query = query_set.q1();

    unsafe {
        for (mut affected_body, e1) in affected_query.iter_unsafe() {
            let mut cumulative_force = Vec2::default();
            let p1 = affected_body.pos;
            let m1 = affected_body.mass;

            for (affecting_body, _) in affecting_query.iter().filter(|(_, e2)| e1 != *e2) {
                let p2 = affecting_body.pos;
                let m2 = affecting_body.mass;

                let rad = p2 - p1;
                let rad_sqr_dist = rad.length_squared();
                let rad_dist = rad_sqr_dist.powf(0.5);

                let current_force = G * m1 * m2 / rad_sqr_dist;

                cumulative_force.x += current_force * rad.x / rad_dist;
                cumulative_force.y += current_force * rad.y / rad_dist;
            }

            affected_body.force += cumulative_force;
        }
    }
}

pub fn collision_sys(
    query_set: QuerySet<(
        Query<(&mut KinematicBody, Entity), Without<Preview>>,
        Query<(&KinematicBody, Entity), Without<Preview>>,
    )>,
    mut commands: Commands,
) {
    use std::collections::HashSet;

    let affected_query = query_set.q0();
    let affecting_query = query_set.q1();

    let mut collided_bodies = HashSet::<Entity>::new();

    unsafe {
        for (mut b1, e1) in affected_query.iter_unsafe() {
            let collided = affecting_query
                .iter()
                .filter(|(_, e2)| e1 != *e2)
                .filter(|(_, e2)| !collided_bodies.contains(e2))
                .filter(|(b2, _)| {
                    let distance_sqr = (b1.pos - b2.pos).length_squared();
                    let total_radius_sqr = (b1.radius + b2.radius).powi(2);

                    distance_sqr <= total_radius_sqr
                });

            let mut e1_has_collided = false;

            let mut total_momentum = b1.mass * b1.vel;
            let mut total_mass = b1.mass;

            for (b2, e2) in collided {
                e1_has_collided = true;

                total_momentum += b2.mass * b2.vel;
                total_mass += b2.mass;

                commands.entity(e2).despawn();
            }

            if e1_has_collided {
                collided_bodies.insert(e1);

                b1.mass = total_mass;
                b1.vel = total_momentum / total_mass;
            }
        }
    }
}
