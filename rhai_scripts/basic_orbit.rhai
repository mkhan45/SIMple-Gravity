       reset_physics();

let sun = add_body(#{
    pos: vec(0.0, 0.0),
    mass: 10_000.0,
    radius: 300.0,
});

let planet = add_body(#{
    pos: vec(4000.0, 0.0),
    vel: vec(0.0, 15.0),
    mass: 0.1,
    radius: 50.0,
});

new_graph("Speed", 1000, 255, 255, 255);
new_graph("Distance", 1000, 255, 0, 0);

let tick = 1;
let max_speed = 1.0;
let max_dist = 1.0;

let update = |a, b| {
    if tick % 10 == 0 && !is_paused() {
        let x = get_vel(planet).x;
        let y = get_vel(planet).y;
        let speed = (x*x + y*y)**(0.5);
        max_speed = max(speed, max_speed);
        add_point("Speed", speed / max_speed);
        
        let dx = get_pos(planet).x - get_pos(sun).x;
        let dy = get_pos(planet).y - get_pos(sun).y;
        let dist = (dx*dx + dy*dy)**(0.5);
        max_dist = max(dist, max_dist);
        add_point("Distance", dist/max_dist);
    }
    tick += 1;
}
