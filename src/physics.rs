use legion::prelude::*;
use crate::{Position, Kinematics, Radius, Mass, main_state::DT, Vector, G};

pub fn integrate_positions(world: &World) {
    let pos_integrate_query = <(Write<Position>, Write<Kinematics>)>::query();
    pos_integrate_query
        .iter(world)
        .for_each(|(pos, kinematics)| {
            pos.0 += kinematics.vel * DT + (kinematics.accel / 2.0) * DT.powi(2);
        });
}

pub fn apply_gravity(world: &World) {
    let gravity_query = <(Read<Position>, Write<Kinematics>, Read<Radius>)>::query();
    let inner_query = <(Read<Position>, Read<Mass>, Read<Radius>)>::query();

    gravity_query
        .iter(world)
        .for_each(|(current_pos, kinematics, rad1)| {
            kinematics.accel = [0.0, 0.0].into();

            inner_query
                .iter(world)
                .for_each(|(other_pos, other_mass, rad2)| {
                    let dist_vec = other_pos.0 - current_pos.0;
                    let dist_mag = current_pos.dist(&other_pos);

                    if dist_mag >= rad1.0 + rad2.0 {
                        let dist_comp = dist_vec / dist_mag;

                        let grav_accel_mag = other_mass.0 / dist_mag.powi(2) * G;
                        let grav_accel: Vector = dist_comp * grav_accel_mag;

                        kinematics.accel += grav_accel
                    }
                });
        });
}

pub fn integrate_kinematics(world: &World) {
    let kinematics_integrate_query = <(Write<Kinematics>)>::query();
    kinematics_integrate_query
        .iter(world)
        .for_each(|kinematics| {
            let vel = &mut kinematics.vel;
            let accel = kinematics.accel;
            let past_accel = &mut kinematics.past_accel;

            *vel += (accel + *past_accel) / 2.0 * DT;

            *past_accel = accel;
        });
}
