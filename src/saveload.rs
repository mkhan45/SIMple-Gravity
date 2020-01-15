use ron;
use specs::{
    error::NoError,
    prelude::*,
    saveload::{SerializeComponents, SimpleMarker},
};

use std::fs::File;
use std::io::{Error, Write};

use crate::ecs::components::{Draw, Kinematics, Mass, Position, Radius, SaveMarker};

pub fn serialize_world(world: &World) -> String {
    let entities = world.entities();
    let positions = world.read_storage::<Position>();
    let kinematics = world.read_storage::<Kinematics>();
    let masses = world.read_storage::<Mass>();
    let draws = world.read_storage::<Draw>();
    let radii = world.read_storage::<Radius>();
    let markers = world.read_storage::<SimpleMarker<SaveMarker>>();

    let mut ser = ron::ser::Serializer::new(Some(Default::default()), true);

    SerializeComponents::<NoError, SimpleMarker<SaveMarker>>::serialize(
        &(&positions, &kinematics, &masses, &draws, &radii),
        &entities,
        &markers,
        &mut ser,
    )
    .expect("error serializing");

    ser.into_output_string()
}

pub fn save_world(world: &World, filename: String) -> Result<(), Error> {
    let mut file = File::create(filename)?;
    file.write_all(serialize_world(world).as_bytes())?;
    Ok(())
}
