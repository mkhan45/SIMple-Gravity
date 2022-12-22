pub const SAMPLE_SCRIPTS: [(&str, &str); 6] = [
    ("Grid", include_str!("../../rhai_scripts/grid.rhai")),
    (
        "Basic Orbit",
        include_str!("../../rhai_scripts/basic_orbit.rhai"),
    ),
    (
        "Binary Star System",
        include_str!("../../rhai_scripts/binary_system.rhai"),
    ),
    (
        "Basic Update",
        include_str!("../../rhai_scripts/basic_update.rhai"),
    ),
    (
        "Scripted Gravity",
        include_str!("../../rhai_scripts/scripted_gravity.rhai"),
    ),
    (
        "Scripted Collisions",
        include_str!("../../rhai_scripts/collisions.rhai"),
    ),
];
