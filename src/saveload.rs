use ron;
use specs::{
    error::NoError,
    prelude::*,
    saveload::{SerializeComponents, SimpleMarker}
};

use crate::ecs::components::{Position, Kinematics, Mass, Draw, Radius, SaveMarker};

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
        &mut ser
        ).expect("error serializing");

    ser.into_output_string()
}
