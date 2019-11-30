use crate::{new_body, Body, Kinematics, Mass, Point, Position, Preview, Radius, Vector, G};
use legion::prelude::*;

use std::collections::HashSet;

pub fn integrate_positions(world: &World, dt: f32) {
    let pos_integrate_query = <(Write<Position>, Write<Kinematics>)>::query();
    pos_integrate_query.par_for_each(world, |(pos, kinematics)| {
        pos.0 += kinematics.vel * dt + (kinematics.accel / 2.0) * dt.powi(2);
    });
}

pub fn apply_gravity(world: &World) {
    //for some reason adding a third component to the query doubles performance
    let gravity_query = <(Read<Position>, Write<Kinematics>, Read<Radius>)>::query();
    let inner_query = <(Read<Position>, Read<Mass>, Read<Radius>)>::query();

    gravity_query.par_for_each(world, |(current_pos, kinematics, _)| {
        kinematics.accel.x = 0.0;
        kinematics.accel.y = 0.0;

        inner_query
            .iter(world)
            .for_each(|(other_pos, other_mass, _)| {
                let dist_vec = other_pos.0 - current_pos.0;
                let dist_mag_sqr = dist_vec.norm_squared();
                let dist_mag = dist_mag_sqr.powf(0.5);

                if current_pos != other_pos {
                    let dist_comp = dist_vec / dist_mag;

                    let grav_accel_mag = other_mass.0 / dist_mag_sqr * G;
                    let grav_accel: Vector = dist_comp * grav_accel_mag;

                    kinematics.accel += grav_accel
                }
            });
    });
}

pub fn integrate_kinematics(world: &World, dt: f32) {
    let kinematics_integrate_query = <(Write<Kinematics>)>::query();
    kinematics_integrate_query.par_for_each(world, |kinematics| {
        let vel = &mut kinematics.vel;
        let accel = kinematics.accel;
        let past_accel = &mut kinematics.past_accel;

        *vel += (accel + *past_accel) / 2.0 * dt;

        *past_accel = accel;
    });
}

pub fn calc_collisions(
    world: &mut World,
    start_point: Option<Point>,
    ctx: &ggez::Context,
    resolution: Vector,
) {
    let collision_query = <(Read<Position>, Read<Radius>, Read<Mass>, Read<Kinematics>)>::query();

    let mut removal_set: HashSet<Entity> = HashSet::new();
    let mut create_set: Vec<Body> = Vec::new();

    collision_query
        .iter_entities(&world)
        .for_each(|(id1, (pos1, r1, m1, k1))| {
            collision_query
                .iter_entities(&world)
                .for_each(|(id2, (pos2, r2, m2, k2))| {
                    if id1 != id2
                        && pos1.dist(*pos2) <= r1.0 + r2.0
                        && !removal_set.contains(&id1)
                        && !removal_set.contains(&id2)
                    {
                        removal_set.insert(id1);
                        removal_set.insert(id2);

                        let p1 = k1.vel * m1.0;
                        let p2 = k2.vel * m2.0;
                        let ptotal = p1 + p2;

                        let mtotal = m1.0 + m2.0;

                        let new_vel = ptotal / mtotal;
                        let new_rad = (r1.0.powi(3) + r2.0.powi(3)).powf(1. / 3.);
                        let new_pos = {
                            let weighted_p1 = pos1.0 * m1.0;
                            let weighted_p2 = pos2.0 * m2.0;
                            let sum_weighted = Point::new(
                                weighted_p1.x + weighted_p2.x,
                                weighted_p1.y + weighted_p2.y,
                            );
                            sum_weighted / mtotal
                        };
                        create_set.push(new_body(new_pos, new_vel, mtotal, new_rad));
                    }
                });
        });

    let collide_preview_query = <(Read<Preview>, Read<Position>, Read<Radius>)>::query();
    let collide_inner_query = <(Read<Position>, Read<Radius>)>::query();

    let mut del_prev_rad: Option<f32> = None;

    collide_preview_query
        .iter_entities(&world)
        .for_each(|(e, (_, p1, r1))| {
            collide_inner_query.iter(&world).for_each(|(p2, r2)| {
                if p1 != p2 && p1.dist(*p2) <= r1.0 + r2.0 {
                    removal_set.insert(e);
                    del_prev_rad = Some(r1.0);
                }
            });
        });

    if let Some(r1) = del_prev_rad {
        if let Some(sp) = start_point {
            let coords = ggez::graphics::screen_coordinates(ctx);
            let mouse_pos = ggez::input::mouse::position(ctx);
            let mouse_pos = crate::main_state::scale_pos(mouse_pos, coords, resolution);
            world.insert_from((), vec![crate::new_preview(sp, (sp - mouse_pos) * 0.1, r1)]);
        }
    }

    removal_set.drain().for_each(|entity| {
        world.delete(entity);
    });

    world.insert_from((), create_set);
}
