use crate::ecs::components::{Position, Preview, Trail};
use crate::ecs::resources::{EnableTrails, RelativeTrails};
use specs::prelude::*;

pub struct TrailSys;

impl<'a> System<'a> for TrailSys {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Trail>,
        Entities<'a>,
        ReadStorage<'a, Preview>,
        Read<'a, EnableTrails>,
        Read<'a, RelativeTrails>,
    );

    fn run(
        &mut self,
        (positions, mut trails, entities, previews, trails_enabled, relative_trails): Self::SystemData,
    ) {
        (&positions, &mut trails, &entities)
            .par_join()
            .for_each(|(pos, trail, entity)| {
                let mut temp_pos = pos.0;
                if relative_trails.enabled {
                    if let Some(rel_pos) = relative_trails.pos {
                        temp_pos.x -= rel_pos.x;
                        temp_pos.y -= rel_pos.y;
                    }
                }
                if previews.get(entity).is_none() {
                    if trails_enabled.0 {
                        trail.points.push_back(temp_pos);
                        while trail.points.len() >= trail.max_len.max(1) {
                            trail.points.pop_front();
                        }
                    } else {
                        trail.points.clear();
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
            .join()
            .for_each(|(pos, trail, _)| {
                trail.points.push_back(pos.0);
            });
    }
}
