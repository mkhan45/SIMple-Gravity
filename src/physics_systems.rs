use specs::prelude::*;

struct PhysicsSys;

impl<'a> System<'a> for PhysicsSys {
    type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Kinematics>, ReadStorage<'a, Radius>, ReadStorage<'a, Mass>);

    fn run(&mut self, (mut pos, mut kine, rad, mass) : Self::SystemData) {
    }
}
