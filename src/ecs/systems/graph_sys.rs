use crate::{Kinematics, SpeedGraph, XVelGraph};
use specs::prelude::*;

// maybe these should all be in one system

macro_rules! impl_graphsys {
    ( $sys:ty, $comp:ty, $access:ident) => {
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
    }
}

pub struct SpeedGraphSys;
fn norm_access(kine: &Kinematics) -> f32 { kine.vel.norm() }
impl_graphsys!(SpeedGraphSys, SpeedGraph, norm_access);

pub struct XVelGraphSys;
fn x_vel_access(kine: &Kinematics) -> f32 { kine.vel.x }
impl_graphsys!(XVelGraphSys, XVelGraph, x_vel_access);

pub struct YVelGraphSys;
fn y_vel_access(kine: &Kinematics) -> f32 { kine.vel.y }
// impl_graphsys!(YVelGraphSys, YVelGraph, y_vel_access);
