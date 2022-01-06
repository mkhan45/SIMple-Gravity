use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::{
    physics::{KinematicBody, Paused, Preview},
    preview::{MultiPreview, PreviewTrailTick},
};

pub struct DrawTrails(pub bool);
pub struct RelativeTrails(pub Option<Entity>);

pub struct Trail {
    pub points: VecDeque<Vec2>,
    pub max_len: usize,
}

impl Default for Trail {
    fn default() -> Self {
        Self {
            points: VecDeque::default(),
            max_len: 125,
        }
    }
}

impl Trail {
    pub fn preview(multi_preview: bool) -> Self {
        Self {
            max_len: if multi_preview { 5_000 } else { 10_000 },
            ..Self::default()
        }
    }
}

pub fn trail_sys(
    mut query_set: QuerySet<(
        Query<(&KinematicBody, Option<&mut Trail>, Entity), Without<Preview>>,
        Query<&KinematicBody>,
    )>,
    mut commands: Commands,
    mut relative_trails_body: ResMut<RelativeTrails>,
    paused: Res<Paused>,
    draw_trails: Res<DrawTrails>,
) {
    if paused.0 || !draw_trails.0 {
        return;
    }

    let relative_pos = {
        let relative_query = query_set.q1();
        relative_trails_body
            .0
            .and_then(|relative_entity| {
                match relative_query.get(relative_entity).map(|body| body.pos) {
                    Ok(pos) => Some(pos),
                    Err(_) => {
                        *relative_trails_body = RelativeTrails(None);
                        None
                    }
                }
            })
            .unwrap_or(Vec2::new(0.0, 0.0))
    };

    let trail_query = query_set.q0_mut();

    for (body, trail, entity) in trail_query.iter_mut() {
        if let Some(mut trail) = trail {
            trail.points.push_back(body.pos - relative_pos);
            while trail.points.len() > trail.max_len {
                trail.points.pop_front();
            }
        } else {
            commands.entity(entity).insert(Trail::default());
        }
    }
}

pub fn clear_trails_sys(
    mut query: Query<&mut Trail, Without<Preview>>,
    draw_trails: Res<DrawTrails>,
    relative_trails_body: Res<RelativeTrails>,
) {
    if (draw_trails.is_changed() && !draw_trails.0) || relative_trails_body.is_changed() {
        for mut trail in query.iter_mut() {
            trail.points.clear();
        }
    }
}

pub fn preview_trail_sys(
    mut query: Query<(&KinematicBody, Option<&mut Trail>, Entity), With<Preview>>,
    mut commands: Commands,
    mut preview_trail_tick: ResMut<PreviewTrailTick>,
    multi_preview: Res<MultiPreview>,
    relative_trails_body: Res<RelativeTrails>,
) {
    if relative_trails_body.0.is_some() {
        return;
    }

    preview_trail_tick.current_tick += 1;
    preview_trail_tick.current_tick %= preview_trail_tick.tick_increment;

    if preview_trail_tick.current_tick == 0 {
        for (body, trail, entity) in query.iter_mut() {
            if let Some(mut trail) = trail {
                if trail.points.len() == trail.max_len {
                    continue;
                }

                trail.points.push_back(body.pos);
                while trail.points.len() > trail.max_len {
                    trail.points.pop_front();
                }
            } else {
                commands
                    .entity(entity)
                    .insert(Trail::preview(multi_preview.0));
            }
        }
    }
}

pub fn draw_trail_sys(
    query: Query<(&KinematicBody, &Trail, Option<&Preview>)>,
    draw_trails: Res<DrawTrails>,
    relative_trails_body: Res<RelativeTrails>,
) {
    if !draw_trails.0 {
        return;
    }

    let relative_pos = {
        relative_trails_body
            .0
            .and_then(|relative_entity| {
                query.get(relative_entity).map(|(body, _, _)| body.pos).ok()
            })
            .unwrap_or(Vec2::new(0.0, 0.0))
    };

    for (body, trail, preview_opt) in query.iter() {
        let points_len = trail.points.len();
        let proportion = |i: usize| i as f32 / points_len as f32;
        for (i, (p1, p2)) in trail
            .points
            .iter()
            .zip(trail.points.iter().skip(1))
            .enumerate()
        {
            let proportion = if preview_opt.is_some() {
                proportion(i) + 0.5
            } else {
                proportion(i)
            };

            let color = Color::new(0.5, 0.7, 1.0, proportion);
            let thickness = proportion * body.radius;

            let p1 = *p1 + relative_pos;
            let p2 = *p2 + relative_pos;
            draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color);
        }
    }
}
