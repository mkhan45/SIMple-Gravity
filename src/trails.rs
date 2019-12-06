use crate::{Position, Preview, Trail, PreviewIterations};
use legion::prelude::*;

pub fn update_trails(world: &mut World) {
    let mut trail_query = <(Read<Position>, Write<Trail>)>::query();
    let mut trail_limited_query = <Write<Trail>>::query().filter(!component::<Preview>());

    trail_query.par_for_each(world, |(pos, mut trail)| {
        trail.0.push_back(pos.0);
    });

    trail_limited_query.par_for_each(world, |mut trail| {
        while trail.0.len() >= 50 {
            trail.0.pop_front();
        }
    });
}
