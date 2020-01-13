use crate::{Kinematics, SpeedGraph};
use specs::prelude::*;

pub struct SpeedGraphSys;

impl<'a> System<'a> for SpeedGraphSys {
    type SystemData = (ReadStorage<'a, Kinematics>, WriteStorage<'a, SpeedGraph>);

    fn run(&mut self, (kinematics, mut graphs): Self::SystemData) {
        (&kinematics, &mut graphs).join().for_each(|(kine, graph)| {
            graph.data.push(kine.vel.norm());

            while graph.data.len() >= 500 {
                graph.data.remove(0);
            }
        });
    }
}

pub struct XVelGraphSys;

impl<'a> System<'a> for XVelGraphSys {
    type SystemData = (ReadStorage<'a, Kinematics>, WriteStorage<'a, SpeedGraph>);

    fn run(&mut self, (kinematics, mut graphs): Self::SystemData) {
        (&kinematics, &mut graphs).join().for_each(|(kine, graph)| {
            graph.data.push(kine.vel.x);

            while graph.data.len() >= 500 {
                graph.data.remove(0);
            }
        });
    }
}
