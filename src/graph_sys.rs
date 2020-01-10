use crate::{Kinematics, SpeedGraph};
use ggez::graphics::{Color, DrawMode, MeshBuilder, Rect};
use specs::prelude::*;

pub struct SpeedGraphSys;

impl<'a> System<'a> for SpeedGraphSys {
    type SystemData = (ReadStorage<'a, Kinematics>, WriteStorage<'a, SpeedGraph>);

    fn run(&mut self, (kinematics, mut graphs): Self::SystemData) {
        (&kinematics, &mut graphs).join().for_each(|(kine, graph)| {
            graph.0.push(kine.vel.norm());

            while graph.0.len() >= 500 {
                graph.0.remove(0);
            }
        });
    }
}
