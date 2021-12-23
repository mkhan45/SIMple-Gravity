use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::{
    body_creation::{CreationData, CreationState},
    camera::CameraRes,
    input_state::MouseState,
    physics::KinematicBody,
};

const PREVIEW_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.75);

pub fn calculate_sides(radius: f32, camera_res: &CameraRes) -> u8 {
    let camera_view_size = (camera_res.camera.screen_to_world(camera_res.screen_size)
        - camera_res.camera.target)
        .abs();
    let screen_proportion = radius * radius / camera_view_size.x / camera_view_size.y;

    let sides = (screen_proportion.cbrt() * 100.0).clamp(10.0, 100.0);
    sides as u8
}

pub fn draw_bodies_sys(query: Query<(&KinematicBody, Option<&Color>)>, camera_res: Res<CameraRes>) {
    for (body, color) in query.iter() {
        let color = color.unwrap_or(&WHITE);

        let sides = calculate_sides(body.radius, &camera_res);
        draw_poly(body.pos.x, body.pos.y, sides, body.radius, 0.0, *color);
    }
}

pub fn draw_create_preview(
    creation_state: Res<CreationState>,
    creation_data: Res<CreationData>,
    camera_res: Res<CameraRes>,
    mouse_state: Res<MouseState>,
) {
    let draw_body_preview = |point: &Vec2| {
        let sides = calculate_sides(creation_data.radius, &camera_res);
        draw_poly(
            point.x,
            point.y,
            sides,
            creation_data.radius,
            0.0,
            PREVIEW_COLOR,
        );
    };

    match *creation_state {
        CreationState::Initiated => {
            draw_body_preview(&mouse_state.prev_position);
        }
        CreationState::Clicked { start_point } => {
            draw_body_preview(&start_point);

            draw_line(
                start_point.x,
                start_point.y,
                mouse_state.prev_position.x,
                mouse_state.prev_position.y,
                (creation_data.radius / 2.0).max(10.0),
                WHITE,
            );
        }
        _ => {}
    }
}