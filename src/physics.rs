use crate::resources::{MainIterations, MousePos, PreviewIterations, Resolution, StartPoint, DT};
use crate::trails::update_trails;
use crate::{new_body, Body, Kinematics, Mass, Point, Position, Preview, Radius, Vector, G};
use legion::prelude::*;
use legion::borrow::{Exclusive, Ref, RefMut, Shared};

type LegionBorrowEx<'a, C> = RefMut<'a, Exclusive<'a>, C>;
type LegionBorrowShare<'a, C> = Ref<'a, Shared<'a>, C>;

use std::collections::HashSet;

pub fn do_physics(world: &mut World, ctx: &mut ggez::Context) {
    let num_iterations = world
        .resources
        .get_or_insert::<MainIterations>(MainIterations(1))
        .unwrap()
        .0;

    let preview_iterations = world
        .resources
        .get_or_insert::<PreviewIterations>(PreviewIterations(25))
        .unwrap()
        .0;

    let mut preview_only = false;
    (0..std::cmp::max(preview_iterations, num_iterations)).for_each(|i| {
        preview_only = i >= num_iterations;
        calc_collisions(world, ctx);
        integrate_positions(world, preview_only);
        apply_gravity(world, preview_only);
        integrate_kinematics(world, preview_only);
        update_trails(world);
    });
}

fn integrate_positions(world: &mut World, preview_only: bool) {
    let dt = world.resources.get_or_insert::<DT>(DT(1.0)).unwrap().0;

    let mut pos_integrate_query = <(Write<Position>, Read<Kinematics>)>::query();

    let int_closure = |(mut pos, kinematics): (
        LegionBorrowEx<'_, Position>,
        LegionBorrowShare<'_, Kinematics>,
    )| {
        pos.0 += kinematics.vel * dt + (kinematics.accel / 2.0) * dt.powi(2);
    };

    // pos_integrate_query.par_for_each(world, |(mut pos, kinematics)| {
    //     pos.0 += kinematics.vel * dt + (kinematics.accel / 2.0) * dt.powi(2);
    // })

    if !preview_only {
        pos_integrate_query.par_for_each(world, int_closure);
    } else {
        pos_integrate_query.filter(component::<Preview>()).par_for_each(world, int_closure);
    }
}

fn apply_gravity(world: &mut World, preview_only: bool) {
    //for some reason adding a third component to the query doubles performance
    let mut gravity_query = <(Read<Position>, Write<Kinematics>, Read<Radius>)>::query();
    let inner_query_vec:
        Vec<(LegionBorrowShare<'_, Position>, 
            LegionBorrowShare<'_, Mass>,
            LegionBorrowShare<'_, Radius>)>
        = <(Read<Position>, Read<Mass>, Read<Radius>)>::query().iter_immutable(&world).collect();

    let grav_closure = |(current_pos, mut kinematics, _) : (
        LegionBorrowShare<'_, Position>,
        LegionBorrowEx<'_, Kinematics>,
        LegionBorrowShare<'_, Radius>,
    ) | {
        // kinematics.accel = inner_query.iter_immutable(&world).fold(
        kinematics.accel = inner_query_vec.iter().fold(
            Vector::new(0.0, 0.0),
            |grav_accel_acc, (other_pos, other_mass, _)| {
                if current_pos != *other_pos {
                    let dist_vec = other_pos.0 - current_pos.0;
                    let dist_mag_sqr = dist_vec.norm_squared();
                    let dist_mag = dist_mag_sqr.powf(0.5);
                    let dist_comp = dist_vec / dist_mag;

                    let grav_accel_mag = other_mass.0 / dist_mag_sqr * G;
                    let grav_accel: Vector = dist_comp * grav_accel_mag;

                    grav_accel_acc + grav_accel
                } else {
                    grav_accel_acc
                }
            },
        );
    };

    unsafe {
        if !preview_only {
            gravity_query.par_for_each_unchecked(world, grav_closure);
        } else {
            gravity_query.filter(component::<Preview>()).par_for_each_unchecked(world, grav_closure);
        }
    }
}

fn integrate_kinematics(world: &mut World, preview_only: bool) {
    let dt = world.resources.get_or_insert::<DT>(DT(1.0)).unwrap().0;
    let mut kinematics_integrate_query = <(Write<Kinematics>)>::query();

    let kine_int_closure = |mut kinematics : (LegionBorrowEx<'_, Kinematics>)| {
        *kinematics.vel = *(kinematics.vel + (kinematics.accel + kinematics.past_accel) / 2.0 * dt);
        kinematics.past_accel = kinematics.accel;
    };

    if !preview_only {
        kinematics_integrate_query.par_for_each(world, kine_int_closure);
    } else {
        kinematics_integrate_query.filter(component::<Preview>()).par_for_each(world, kine_int_closure);
    }
    // *kinematics.vel = *(kinematics.vel + (kinematics.accel + kinematics.past_accel) / 2.0 * dt);
    // kinematics.past_accel = kinematics.accel;
}


//TODO Make this accept preview_only
fn calc_collisions(world: &mut World, ctx: &ggez::Context) {
    let start_point = world
        .resources
        .get::<StartPoint>()
        .expect("error getting start point")
        .clone();
    let resolution = world
        .resources
        .get::<Resolution>()
        .expect("error getting resolution")
        .clone();

    let mut collision_query =
        <(Read<Position>, Read<Radius>, Read<Mass>, Read<Kinematics>)>::query();

    let mut removal_set: HashSet<Entity> = HashSet::new();
    let mut create_set: Vec<Body> = Vec::new();

    collision_query
        .clone()
        .iter_entities_immutable(world)
        .for_each(|(id1, (pos1, r1, m1, k1))| {
            collision_query
                .iter_entities_immutable(world)
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

    let mut collide_preview_query = <(Read<Preview>, Read<Position>, Read<Radius>)>::query();
    let mut collide_inner_query = <(Read<Position>, Read<Radius>)>::query();

    let mut del_prev_rad: Option<f32> = None;

    collide_preview_query
        .iter_entities_immutable(world)
        .for_each(|(e, (_, p1, r1))| {
            collide_inner_query
                .iter_immutable(world)
                .for_each(|(p2, r2)| {
                    if p1 != p2 && p1.dist(*p2) <= r1.0 + r2.0 {
                        removal_set.insert(e);
                        del_prev_rad = Some(r1.0);
                    }
                });
        });

    if let Some(r1) = del_prev_rad {
        if let Some(sp) = start_point.0 {
            let coords = ggez::graphics::screen_coordinates(ctx);
            let mouse_pos = ggez::input::mouse::position(ctx);
            let mouse_pos = crate::main_state::scale_pos(mouse_pos, coords, resolution.0);
            world.insert((), vec![crate::new_preview(sp, (sp - mouse_pos) * 0.1, r1)]);
        }
    }

    removal_set.drain().for_each(|entity| {
        world.delete(entity);
    });

    world.insert((), create_set);
}
