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

let update = |ids, bodies| {
    draw_spring(vec(0.0, 0.0), vec(000.0, 5000.0), 500.0, 25.0, 10);
};
