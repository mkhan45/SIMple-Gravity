use crate::{Position, Preview, Trail};
use specs::prelude::*;

pub struct TrailSys;

impl<'a> System<'a> for TrailSys {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Trail>,
        Entities<'a>,
        ReadStorage<'a, Preview>,
    );

    fn run(&mut self, (positions, mut trails, entities, previews): Self::SystemData) {
        (&positions, &mut trails, &entities)
            .par_join()
            .for_each(|(pos, trail, entity)| {
                if previews.get(entity).is_none() {
                    trail.0.push_back(pos.0);

                    while trail.0.len() >= 35 {
                        trail.0.pop_front();
                    }
                }
            });
    }
}

pub struct PreviewTrailSys;

impl<'a> System<'a> for PreviewTrailSys {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Trail>,
        ReadStorage<'a, Preview>,
    );

    fn run(&mut self, (positions, mut trails, previews): Self::SystemData) {
        (&positions, &mut trails, &previews)
            .par_join()
            .for_each(|(pos, trail, _)| {
                trail.0.push_back(pos.0);
            });
    }
}
