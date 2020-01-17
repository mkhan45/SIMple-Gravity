use ron;
use specs::{
    error::NoError,
    prelude::*,
    saveload::{SerializeComponents, SimpleMarker, DeserializeComponents, SimpleMarkerAllocator},
};

use std::fs::File;
use std::io::{Error, Write, Read};
use std::fmt;

use crate::ecs::components::{Draw, Kinematics, Mass, Position, Radius, SaveMarker};

// https://github.com/amethyst/specs/blob/master/examples/saveload.rs

#[derive(Debug)]
enum ComboError {
    Ron(ron::ser::Error),
}

impl From<ron::ser::Error> for ComboError {
    fn from(x: ron::ser::Error) -> Self {
        ComboError::Ron(x)
    }
}

impl From<NoError> for ComboError {
    fn from(e: NoError) -> Self {
        match e {}
    }
}

impl fmt::Display for ComboError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ComboError::Ron(ref e) => write!(f, "{}", e),
        }
    }
}

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

pub fn load_world(world: &World, filename: String) -> Result<(), Error> {
    let entities = world.entities();
    let positions = world.write_storage::<Position>();
    let kinematics = world.write_storage::<Kinematics>();
    let masses = world.write_storage::<Mass>();
    let draws = world.write_storage::<Draw>();
    let radii = world.write_storage::<Radius>();
    let mut markers = world.write_storage::<SimpleMarker<SaveMarker>>();
    let mut alloc = world.write_resource::<SimpleMarkerAllocator<SaveMarker>>();

    use ron::de::Deserializer;

    let mut file = File::open(filename)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    if let Ok(mut de) = Deserializer::from_str(&file_contents) {
        DeserializeComponents::<ComboError, _>::deserialize(
            &mut (positions, kinematics, masses, draws, radii),
            &entities,
            &mut markers,
            &mut alloc,
            &mut de
        ).unwrap_or_else(|e| eprintln!("Error: {}", e));
    }

    Ok(())
}
