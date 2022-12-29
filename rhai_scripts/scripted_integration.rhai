set_integration(false);

let update = |ids, bodies| {
    for body in ids {
        let pos = body.get_pos();
        let vel = body.get_vel();
        let accel = body.get_accel();
        let force = body.get_force();
        let mass = body.get_mass();

        body.set_pos(pos + vel);
        body.set_vel(vel + accel);
        body.set_accel(force / mass);
        body.set_force(vec(0, 0));
    }
};
