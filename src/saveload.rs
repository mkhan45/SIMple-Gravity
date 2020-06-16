use specs::prelude::*;

use std::io::Error;

use crate::ecs::components::{Kinematics, Mass, Position, Radius, Trail};

pub fn save_world_to_lua(world: &World, filename: String) -> Result<(), Error> {
    let mut body_string = String::with_capacity(48);

    let positions = world.read_storage::<Position>();
    let kinematics = world.read_storage::<Kinematics>();
    let radii = world.read_storage::<Radius>();
    let masses = world.read_storage::<Mass>();

    let mut first = true;

    (&positions, &kinematics, &radii, &masses).join().for_each(|(pos, kine, rad, mass)|{
        if !first {
            body_string.push_str(",\n\t");
        } else {
            first = false;
        }

        body_string.push_str(
            format!(
                "{{x = {x:.prec$}, y = {y:.prec$}, x_vel = {x_vel:.prec$}, y_vel = {y_vel:.prec$}, mass = {mass:.prec$}, rad = {rad:.prec$}}}",
                x = pos.0.x,
                y = pos.0.y,
                x_vel = kine.vel.x,
                y_vel = kine.vel.y,
                mass = mass.0,
                rad = rad.0,
                prec = 3,
                ).as_str())
    });

    std::fs::write(filename, format!("add_bodies(\n\t{}\n)", body_string)).unwrap();

    Ok(())
}

pub fn load_from_lua(world: &World, filename: String) -> Result<(), Error> {
    let lua = world.fetch_mut::<crate::ecs::resources::LuaRes>().clone();

    lua.lock().unwrap().context(|lua_ctx| {
        let lua_code = std::fs::read_to_string(filename.clone()).unwrap();
        if let Err(e) = lua_ctx
            .load(&lua_code)
            .set_name(filename.as_str())
            .unwrap()
            .exec()
        {
            println!("Lua {}", e.to_string());
        };
    });

    Ok(())
}

pub fn load_world(world: &World, filename: String) -> Result<(), Error> {
    {
        let mut trails = world.write_storage::<Trail>();

        (&mut trails).join().for_each(|trail| {
            trail.points.clear();
        });
    }

    match &filename.as_str()[filename.len() - 4..] {
        ".lua" => load_from_lua(world, filename),
        _ => panic!("invalid file"),
    }?;

    Ok(())
}
