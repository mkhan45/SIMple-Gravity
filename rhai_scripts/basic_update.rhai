reset_physics();

// add a body and sets b as its ID.
let b = add_body(#{
    radius: 200.0, 
    vel: vec(5.0, 15.0),
    // bodies must be registered to be scriptable
    registered: true, 
});

// called every frame
// make sure this is a closure instead of a a function,
// or else it will not have access to existing variables
let update = |ids, bodies| {
    if get_body(b) != () {
        if get_pos(b).x > 500.0 {
            b.add_force(vec(-20.0, 0.0));
        } else if get_pos(b).x < -500.0 {
            b.add_force(vec(20.0, 0.0));
        }
        
        if get_pos(b).y > 500.0 {
            b.add_force(vec(0.0, -20.0));
        } else if get_pos(b).y < -500.0 {
            b.add_force(vec(0.0, 20.0));
        }
    }
}
