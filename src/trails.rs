use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::{KinematicBody, Paused};

#[derive(Debug)]
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

pub fn trail_sys(
    mut query: Query<(&KinematicBody, Option<&mut Trail>, Entity)>,
    mut commands: Commands,
    paused: Res<Paused>,
) {
    if paused.0 {
        return
    }

    for (body, trail, entity) in query.iter_mut() {
        if let Some(mut trail) = trail {
            trail.points.push_back(body.pos);
            while trail.points.len() > trail.max_len {
                trail.points.pop_front();
            }
        } else {
            commands.entity(entity).insert(Trail::default());
        }
    }
}

pub fn draw_trail_sys(query: Query<(&KinematicBody, &Trail)>) {
    for (body, trail) in query.iter() {
        let points_len = trail.points.len();
        let proportion = |i: usize| i as f32 / points_len as f32;
        for (i, (p1, p2)) in trail
            .points
            .iter()
            .zip(trail.points.iter().skip(1))
            .enumerate()
        {
            let proportion = proportion(i);
            let color = Color::new(0.5, 0.7, 1.0, proportion);
            let thickness = proportion * body.radius;
            draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color);
        }
    }
}
