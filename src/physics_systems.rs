use specs::prelude::*;

use crate::components::{Kinematics, Mass, Position, Radius, Preview};
use crate::resources::{MainIterations, PreviewIterations, StartPoint, DT};

pub struct PhysicsSys;

impl<'a> System<'a> for PhysicsSys {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Preview>,
        ReadStorage<'a, Radius>,
        ReadStorage<'a, Mass>,
        Read<'a, MainIterations>,
        Read<'a, PreviewIterations>,
        Read<'a, StartPoint>,
        Read<'a, DT>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut kinematics,
            previews,
            radii,
            masses,
            main_iterations,
            preview_iterations,
            start_point,
            dt,
        ): Self::SystemData,
    ) {
        dbg!("main");
        let mut preview_only = false;
        (0..std::cmp::max(preview_iterations.0, main_iterations.0))
            .for_each(|i| {
                preview_only = i >= main_iterations.0;
                integrate_positions(&mut positions, &kinematics, &previews, preview_only, dt.0);
            });
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

    let int_closure = |(pos, kinematics): (
        &mut Position,
        &Kinematics,
    )| {
        dbg!("positions");
        pos.0 += kinematics.vel * dt + (kinematics.accel / 2.0) * dt.powi(2);
    };

    if !preview_only{
        (positions, kinematics).join().for_each(int_closure);
    } else {
        (positions, kinematics, previews).join().for_each(|(pos, kine, _)|{
            int_closure((pos, kine));
        });
    }
}
