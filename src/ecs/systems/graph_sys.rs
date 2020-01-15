use crate::ecs::components::{Kinematics, SpeedGraph, XVelGraph, YVelGraph};
use specs::prelude::*;

// To add a new type of graph:
// 1. Make a new graph component in components.rs with make_graph_components
// 2. Make a new graph system here
// 3. Add a graph button to the GUI
// 4. In main_state.rs register the graph data with register_graph_data!
// 5. Add the graph to undisplay_graphs! and add_graphs!
// TODO: Automate the last 2 (3?) steps

macro_rules! make_graphsys {
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

fn norm_access(kine: &Kinematics) -> f32 {
    kine.vel.norm()
}
make_graphsys!(SpeedGraphSys, SpeedGraph, norm_access);

fn x_vel_access(kine: &Kinematics) -> f32 {
    kine.vel.x
}
make_graphsys!(XVelGraphSys, XVelGraph, x_vel_access);

fn y_vel_access(kine: &Kinematics) -> f32 {
    kine.vel.y
}
make_graphsys!(YVelGraphSys, YVelGraph, y_vel_access);
