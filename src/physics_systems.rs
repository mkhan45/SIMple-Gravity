use specs::prelude::*;

use crate::components::{Kinematics, Mass, Position, Preview, Radius, Draw, Trail};
use crate::resources::{DT, NewPreview};
use crate::{new_body, Body, Point, Vector, G};

use std::collections::HashSet;

pub struct PhysicsSys;

impl<'a> System<'a> for PhysicsSys {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Preview>,
        WriteStorage<'a, Radius>,
        WriteStorage<'a, Mass>,
        Entities<'a>,
        Read<'a, DT>,
        WriteStorage<'a, Draw>,
        WriteStorage<'a, Trail>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut kinematics,
            mut previews,
            mut radii,
            mut masses,
            entities,
            dt,
            mut draws,
            mut trails,
        ): Self::SystemData,
    ) {
        integrate_positions(&mut positions, &kinematics, &previews, false, dt.0);
        apply_gravity(
            &positions,
            &mut kinematics,
            &radii,
            &masses,
            &previews,
            false,
        );
        integrate_kinematics(&mut kinematics, &previews, false, dt.0);
        let (mut c_vec, mut delete_set) =
            calc_collisions(&positions, &kinematics, &masses, &radii, &entities);

        delete_set.drain().for_each(|e|{
            entities.delete(e).expect("error deleting collided entity");
        });

        c_vec.drain(..).for_each(|body|{
            entities.build_entity()
                .with(body.0, &mut positions)
                .with(body.1, &mut kinematics)
                .with(body.2, &mut masses)
                .with(body.3, &mut draws)
                .with(body.4, &mut radii)
                .with(body.5, &mut trails)
                .build();
        });
        // create_vec.0 = c_vec;
        // del_set.0 = delete_set;
    }
}

pub struct PreviewPhysicsSys;

impl<'a> System<'a> for PreviewPhysicsSys {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Preview>,
        WriteStorage<'a, Radius>,
        WriteStorage<'a, Mass>,
        Read<'a, DT>,
        Write<'a, NewPreview>,
    );

    fn run(
        &mut self,
        (mut positions, mut kinematics, previews, radii, masses, dt, mut new_preview): Self::SystemData,
    ) {
        integrate_positions(&mut positions, &kinematics, &previews, true, dt.0);
        apply_gravity(
            &positions,
            &mut kinematics,
            &radii,
            &masses,
            &previews,
            true,
        );

        (&positions, &radii).join().for_each(|(pos1, rad1)|{
            (&positions, &radii).join().for_each(|(pos2, rad2)|{
                if pos1 != pos2 && pos1.dist(pos2.0) <= rad1.0 + rad2.0 {
                    new_preview.0 = true;
                }
            });
        });

        integrate_kinematics(&mut kinematics, &previews, true, dt.0);
    }
}

fn integrate_positions(
    positions: &mut WriteStorage<'_, Position>,
    kinematics: &WriteStorage<'_, Kinematics>,
    previews: &ReadStorage<'_, Preview>,
    preview_only: bool,
    dt: f32,
) {
    // let mut pos_integrate_query = <(Write<Position>, Read<Kinematics>)>::query();

    let int_closure = |(pos, kinematics): (&mut Position, &Kinematics)| {
        pos.0 += kinematics.vel * dt + (kinematics.accel / 2.0) * dt.powi(2);
    };

    if !preview_only {
        (positions, kinematics).par_join().for_each(int_closure);
    } else {
        (positions, kinematics, previews)
            .join()
            .for_each(|(pos, kine, _)| {
                int_closure((pos, kine));
            });
    }
}

fn apply_gravity(
    positions: &WriteStorage<'_, Position>,
    kinematics: &mut WriteStorage<'_, Kinematics>,
    radii: &WriteStorage<'_, Radius>,
    masses: &WriteStorage<'_, Mass>,
    previews: &ReadStorage<'_, Preview>,
    preview_only: bool,
) {
    let grav_closure = |(current_pos, kinematics, _): (&Position, &mut Kinematics, &Radius)| {
        // kinematics.accel = inner_query.iter_immutable(&world).fold(
        kinematics.accel = (positions, masses).join().fold(
            Vector::new(0.0, 0.0),
            |grav_accel_acc, (other_pos, other_mass)| {
                if current_pos != other_pos {
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

    if !preview_only {
        (positions, kinematics, radii)
            .par_join()
            .for_each(grav_closure);
        } else {
            (positions, kinematics, radii, previews)
                .join()
                .for_each(|(p, k, r, _)| grav_closure((p, k, r)));
    }
}

fn integrate_kinematics(
    kinematics: &mut WriteStorage<'_, Kinematics>,
    previews: &ReadStorage<'_, Preview>,
    preview_only: bool,
    dt: f32,
) {
    let kine_int_closure = |kinematics: &mut Kinematics| {
        *kinematics.vel = *(kinematics.vel + (kinematics.accel + kinematics.past_accel) / 2.0 * dt);
        kinematics.past_accel = kinematics.accel;
    };

    if !preview_only {
        (kinematics).par_join().for_each(kine_int_closure);
    } else {
        (kinematics, previews).join().for_each(|(kine, _)| {
            kine_int_closure(kine);
        });
    }
}

fn calc_collisions(
    positions: &WriteStorage<'_, Position>,
    kinematics: &WriteStorage<'_, Kinematics>,
    masses: &WriteStorage<'_, Mass>,
    radii: &WriteStorage<'_, Radius>,
    entities: &Entities,
) -> (Vec<Body>, HashSet<Entity>) {
    let mut create_vec: Vec<Body> = Vec::new();
    let mut delete_set: HashSet<Entity> = HashSet::new();

    (positions, radii, masses, kinematics, entities)
        .join()
        .for_each(|(pos1, r1, m1, k1, e1)| {
            (positions, radii, masses, kinematics, entities)
                .join()
                .for_each(|(pos2, r2, m2, k2, e2)| {
                    if e1 != e2
                        && pos1.dist(*pos2) <= r1.0 + r2.0
                            && !delete_set.contains(&e1)
                            && !delete_set.contains(&e2)
                    {
                        delete_set.insert(e1);
                        delete_set.insert(e2);

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
                        create_vec.push(new_body(new_pos, new_vel, mtotal, new_rad));
                    }
                });
        });

    (create_vec, delete_set)
}
