const SPRING_CONSTANT = 0.001;
const DAMPING_FACTOR = 0.999;

fn apply_spring_forces(ids) {
    for body in ids {
        draw_spring(
            vec(0.0, body.get_pos().y),
            body.get_pos(),
            body.get_radius() * 2.0,
            body.get_radius() / 2.0,
            10
        );
        let spring_length = (body.get_pos().x + body.get_vel().x);
        body.add_force(vec(SPRING_CONSTANT * -spring_length, 0.0));
        body.set_vel(body.get_vel() * DAMPING_FACTOR);
    }
}

set_g(0.0);

new_graph("Kinetic", 1000, 0, 0, 255);
new_graph("Potential", 1000, 255, 0, 0);
new_graph("Total", 1000, 0, 255, 0);


let update = |ids, bodies| {
    if (!is_paused()) {
        apply_spring_forces(ids);
    }

    let total_kinetic_energy = 0.0;
    let total_potential_energy = 0.0;
    for body in ids {
        let m = body.get_mass();
        let x = body.get_pos().x;
        let v = body.get_vel();
        let v_mag = v.length;

        // use the SPRING_CONSTANT variable for 
        // potential energy
        let body_kinetic_energy = 0.5 * m * v_mag**2;
        let body_potential_energy = 0.5 * SPRING_CONSTANT * x**2;
        
        if body == ids[0] {
            add_point("Kinetic", body_kinetic_energy);
            add_point("Potential", body_potential_energy);
            add_point("Total", body_kinetic_energy + body_potential_energy);
        }

        // add the body's energy to the total
        total_kinetic_energy += body_kinetic_energy;
        total_potential_energy += body_potential_energy;
    }

    let total_energy = 
        total_kinetic_energy + total_potential_energy;
};

fn draw_spring(start, end, width, thickness, coils) {
    let r = end - start;
    let incr = r / coils;
    let offs = incr.rotate(3.1415 / 2.0).normalized * width;

    let pos = start;
    for i in 0..coils {
        let p1 = pos + (incr / 3.0) + (offs / 2.0);
        let p2 = pos + (incr * 2.0/3.0) - (offs / 2.0);
        let p3 = pos + incr;
        draw_line(pos, p1, thickness);
        draw_line(p1, p2, thickness);
        draw_line(p2, p3, thickness);
        pos = p3;
    }
}