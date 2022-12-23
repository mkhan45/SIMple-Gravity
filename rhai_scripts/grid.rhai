reset_physics();

for i in range(0, 10) {
    for j in range(0, 10) {
        add_body(#{pos: vec(i * 300.0, j * 300.0), radius: 50.0, mass: 3.0});
    }
}
