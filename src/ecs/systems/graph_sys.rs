use crate::ecs::components::{Kinematics, SpeedGraph, XVelGraph, YVelGraph};
use specs::prelude::*;

// maybe these should all be in one system

macro_rules! new_graphsys {
    ( $sys:ident, $comp:ty, $access:ident) => {
        pub struct $sys;
        impl<'a> System<'a> for $sys {
            type SystemData = (ReadStorage<'a, Kinematics>, WriteStorage<'a, $comp>);

            fn run(&mut self, (kinematics, mut graphs): Self::SystemData) {
                (&kinematics, &mut graphs).join().for_each(|(kine, graph)| {
                    graph.data.push($access(kine));

                    while graph.data.len() >= 500 {
                        graph.data.remove(0);
                    }
                });
            }
        }
    };
}

#[rustfmt::skip]
fn norm_access(kine: &Kinematics) -> f32 { kine.vel.norm() }
new_graphsys!(SpeedGraphSys, SpeedGraph, norm_access);

#[rustfmt::skip]
fn x_vel_access(kine: &Kinematics) -> f32 { kine.vel.x }
new_graphsys!(XVelGraphSys, XVelGraph, x_vel_access);

#[rustfmt::skip]
fn y_vel_access(kine: &Kinematics) -> f32 { kine.vel.y }
new_graphsys!(YVelGraphSys, YVelGraph, y_vel_access);
