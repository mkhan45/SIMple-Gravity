use specs::prelude::*;

use crate::components::{Kinematics, Mass, Position, Radius};
use crate::resources::{MainIterations, PreviewIterations, StartPoint};

struct PhysicsSys;

impl<'a> System<'a> for PhysicsSys {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Radius>,
        ReadStorage<'a, Mass>,
        Read<'a, MainIterations>,
        Read<'a, PreviewIterations>,
        Read<'a, StartPoint>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut kinematics,
            radii,
            masses,
            main_iterations,
            preview_iterations,
            start_point,
        ): Self::SystemData,
    ) {
        let mut preview_only = false;
        (0..std::cmp::max(preview_iterations.0, main_iterations.0))
            .for_each(|i| preview_only = i >= main_iterations.0);
    }
}
