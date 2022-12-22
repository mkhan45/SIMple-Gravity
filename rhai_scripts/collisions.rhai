set_collisions(false);

let ELASTICITY = 1.0;

let update = |ids, bodies| {
    let nc = 0;
    for i in 0..(ids.length - 1) {
        for j in (i+1)..ids.length {
            let a = ids[i];
            let b = ids[j];
            
            let m1 = a.get_mass();
            let m2 = b.get_mass();
            let mt = m1 + m2;

            let v1 = a.get_vel();
            let v2 = b.get_vel();

            let r1 = a.get_radius();
            let r2 = b.get_radius();
            let r = b.get_pos() - a.get_pos();
            let nr = (b.get_pos() + v2) - (a.get_pos() + v1);

            // check if the bodies are touching and they are actually
            // going towards each other
            if r.length <= (r1 + r2) && nr.length <= r.length {
                nc += 1;
                let v1f = ((m1 - m2) / mt) * v1 + ((2 * m2) / mt) * v2;
                let v2f = ((2 * m1) / mt) * v1 + ((m2 - m1) / mt) * v2;

                a.set_vel(v1f);
                b.set_vel(v2f);
            }
        }
    }
};
