reset_physics();
// set engine gravity to 0
set_g(0.0);

const G = 100.0;

let update = |ids, bodies| {
    for i in 0..(ids.length - 1) {
        for j in (i+1)..ids.length {
            let a = ids[i];
            let b = ids[j];
            let m1 = a.get_mass();
            let m2 = b.get_mass();
            let r = b.get_pos() - a.get_pos();

            let F = G * m1 * m2 / (r.length * r.length);
            let F_x = F * r.x / r.length;
            let F_y = F * r.y / r.length;

            //println(F_x);
            a.add_force(vec(F_x, F_y));
            b.add_force(vec(-F_x, -F_y));
        }
    }
};

