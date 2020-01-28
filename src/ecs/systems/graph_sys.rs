use crate::ecs::components::{AccelGraph, Kinematics, SpeedGraph, XVelGraph, YVelGraph};
use crate::Vector;
use specs::prelude::*;

// To add a new type of graph:
// 1. Make a new graph component in components.rs with make_graph_components
// 2. Make a new graph system here
// 3. Add a graph button to the GUI
// 4. In draw_utils.rs register the graph data with register_graph_data!
// 5. Add the graph to undisplay_graphs! and add_graphs!
// TODO: Automate the last 2 (3?) steps
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GraphType {
    Speed,
    XVel,
    YVel,
    Accel,
}

macro_rules! make_graphsys {
    ( $sys:ident, $comp:ty, $access:ident, $graph_len:expr) => {
        pub struct $sys;
        impl<'a> System<'a> for $sys {
            type SystemData = (ReadStorage<'a, Kinematics>, WriteStorage<'a, $comp>);

            fn run(&mut self, (kinematics, mut graphs): Self::SystemData) {
                (&kinematics, &mut graphs).join().for_each(|(kine, graph)| {
                    graph.data.push($access(kine));

                    while graph.data.len() >= $graph_len {
                        graph.data.remove(0);
                    }
                });
            }
        }
    };
}

fn norm_access(kine: &Kinematics) -> f32 {
    kine.vel.norm()
}
make_graphsys!(SpeedGraphSys, SpeedGraph, norm_access, 500);

fn x_vel_access(kine: &Kinematics) -> f32 {
    kine.vel.x
}
make_graphsys!(XVelGraphSys, XVelGraph, x_vel_access, 500);

fn y_vel_access(kine: &Kinematics) -> f32 {
    kine.vel.y
}
make_graphsys!(YVelGraphSys, YVelGraph, y_vel_access, 500);

fn accel_access(kine: &Kinematics) -> Vector {
    kine.accel
}
make_graphsys!(AccelGraphSys, AccelGraph, accel_access, 25);
