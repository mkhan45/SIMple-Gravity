// set engine gravity to 0
set_g(0.0);

const G = 100.0;

let sun = add_body(#{
    radius: 300.0,
    mass: 10_000.0,
    registered: true,
});

let earth = add_body(#{
    pos: vec(4000.0, 0.0),
    mass: 0.1,
    radius: 50.0,
    registered: true,
    vel: vec(0.0, 15.0),
});

let update = || {
    // make sure earth is properly instantiated,
    // creating bodies takes one tick
    if earth.get_body() != () {
        let m1 = earth.get_mass();
        let m2 = sun.get_mass();
        let r = (sun.get_pos() - earth.get_pos());

        let F = G * m1 * m2 / (r.length * r.length);
        let F_x = F * r.x / r.length;
        let F_y = F * r.y / r.length;

        earth.add_force(vec(F_x, F_y));
    }
};
