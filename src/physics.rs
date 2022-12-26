use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::{
    scripting::{RhaiBody, RhaiRes},
    trails::Trail,
    ui::inspect::InspectedEntity,
};

pub struct DT(pub f32);
pub struct Steps(pub usize);
pub struct G(pub f32);

pub struct PhysicsToggles {
    pub collisions: bool,
    pub integration: bool,
}

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

pub struct Paused(pub bool);

macro_rules! generate_integration_systems {
    ($filter:ident, $name:ident) => {
        pub fn $name(
            mut query: Query<&mut KinematicBody, $filter<Preview>>,
            dt: Res<DT>,
            physics_toggles: Res<PhysicsToggles>,
            paused: Res<Paused>,
        ) {
            if paused.0 || !physics_toggles.integration {
                return;
            }

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
    g: Res<G>,
    paused: Res<Paused>,
) {
    if paused.0 {
        return;
    }

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

                let current_force = g.0 * m1 * m2 / rad_sqr_dist;

                cumulative_force.x += current_force * rad.x / rad_dist;
                cumulative_force.y += current_force * rad.y / rad_dist;
            }

            affected_body.force += cumulative_force;
        }
    }
}

pub fn collision_sys(
    query_set: QuerySet<(
        Query<(&mut KinematicBody, Option<&mut Trail>, Entity), Without<Preview>>,
        Query<(&KinematicBody, Entity), Without<Preview>>,
    )>,
    mut inspected_entity: ResMut<InspectedEntity>,
    mut commands: Commands,
    _rhai_bodies: Query<&RhaiBody>,
    paused: Res<Paused>,
    physics_toggles: Res<PhysicsToggles>,
    _rhai: Res<RhaiRes>,
) {
    use std::collections::HashSet;

    if paused.0 || !physics_toggles.collisions {
        return;
    }

    let affected_query = query_set.q0();
    let affecting_query = query_set.q1();

    let mut collided_bodies = HashSet::<Entity>::new();

    unsafe {
        for (mut b1, mut trail, e1) in affected_query.iter_unsafe() {
            if collided_bodies.contains(&e1) {
                continue;
            }

            let collided = affecting_query
                .iter()
                .filter(|(_, e2)| e1 != *e2)
                .filter(|(b2, _)| {
                    let distance_sqr = (b1.pos - b2.pos).length_squared();
                    let total_radius_sqr = (b1.radius + b2.radius).powi(2);

                    distance_sqr <= total_radius_sqr
                });

            let mut e1_has_collided = false;

            let mut total_momentum = b1.mass * b1.vel;
            let mut total_mass = b1.mass;
            let mut total_volume = b1.radius.powi(3);
            let mut total_moment = b1.pos * b1.mass;
            let mut total_force = b1.force;

            let mut inspected_is_collided = false;

            for (b2, e2) in collided {
                if collided_bodies.contains(&e2) {
                    continue;
                } else {
                    collided_bodies.insert(e2);
                }

                e1_has_collided = true;

                total_momentum += b2.mass * b2.vel;
                total_mass += b2.mass;
                total_volume += b2.radius.powi(3);
                total_moment += b2.pos * b2.mass;
                total_force += b2.force;

                // if let Ok(rhai_id) = rhai_bodies.get(e2) {
                //     rhai.existing_bodies.write().unwrap().
                // }
                commands.entity(e2).despawn();

                if inspected_entity.0.contains(&e2) {
                    inspected_is_collided = true;
                }
            }

            if e1_has_collided {
                if let Some(Trail { points, .. }) = trail.as_deref_mut() {
                    points.clear();
                }

                collided_bodies.insert(e1);

                b1.mass = total_mass;
                b1.vel = total_momentum / total_mass;
                b1.radius = total_volume.powf(1.0 / 3.0);

                b1.pos = total_moment / total_mass;
                b1.force = total_force;
                b1.accel = total_force / total_mass;

                if inspected_is_collided {
                    inspected_entity.0 = Some(e1);
                }
            }
        }
    }
}
