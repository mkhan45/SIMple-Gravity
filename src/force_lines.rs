use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::{KinematicBody, Preview};

pub struct DrawForceLines(pub bool);

pub struct ForceLine {
    pub end_point: Vec2,
    pub magnitude: f32,
    pub max_width: f32,
}

pub fn force_line_sys(
    mut force_line_query: Query<(&KinematicBody, Option<&mut Vec<ForceLine>>, Entity), Without<Preview>>,
    body_query: Query<(&KinematicBody, Entity), Without<Preview>>,
    draw_force_lines: Res<DrawForceLines>,
    mut commands: Commands,
) {
    if draw_force_lines.0 {
        for (body, force_lines, entity) in force_line_query.iter_mut() {
            if let Some(mut force_lines) = force_lines {
                force_lines.clear();
                for (other_body, _) in body_query.iter().filter(|(_, e)| *e != entity) {
                    let radius_sqr = (other_body.pos - body.pos).length_squared();

                    force_lines.push(ForceLine {
                        magnitude: other_body.mass / radius_sqr,
                        end_point: other_body.pos,
                        max_width: body.radius.min(other_body.radius) * 0.5,
                    });
                }
            } else {
                commands.entity(entity).insert(Vec::<ForceLine>::new());
            }
        }
    }
}
