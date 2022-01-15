use bevy_ecs::prelude::*;
use egui_macroquad::{egui::CtxRef, macroquad};
use macroquad::prelude::*;

use super::input_state::MouseState;
use crate::{
    camera::CameraRes,
    physics::{KinematicBody, Preview},
    preview::MultiPreview,
};

#[derive(PartialEq, Debug)]
pub enum CreationState {
    Unstarted,
    Initiated,
    Clicked { start_point: Vec2 },
}

pub struct CreationData {
    pub radius: f32,
    pub mass: f32,
}

impl Default for CreationData {
    fn default() -> Self {
        Self {
            radius: 75.0,
            mass: 1.0,
        }
    }
}

pub fn create_body_sys(
    mut creation_state: ResMut<CreationState>,
    mut creation_data: ResMut<CreationData>,
    mouse_state: Res<MouseState>,
    mut commands: Commands,
    preview_query: Query<Entity, With<Preview>>,
    multi_preview: Res<MultiPreview>,
    camera_res: Res<CameraRes>,
    egui_ctx: Res<CtxRef>,
) {
    match *creation_state {
        CreationState::Unstarted => {
            if is_key_pressed(KeyCode::C) {
                *creation_state = CreationState::Initiated;
            }
        }
        CreationState::Initiated => {
            if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::Escape) {
                *creation_state = CreationState::Unstarted;
            }

            if is_mouse_button_pressed(MouseButton::Left) && !egui_ctx.is_pointer_over_area() {
                *creation_state = CreationState::Clicked {
                    start_point: mouse_state.prev_position,
                }
            }

            if is_key_down(KeyCode::LeftShift) && mouse_wheel().1 != 0.0 {
                let scale = 1.0 + mouse_wheel().1.signum() * -0.1;
                creation_data.radius *= scale;
            }
        }
        CreationState::Clicked { start_point } => {
            if is_mouse_button_released(MouseButton::Left) && !egui_ctx.is_pointer_over_area() {
                if !multi_preview.0 {
                    preview_query.iter().for_each(|entity| {
                        commands.entity(entity).despawn();
                    });
                }

                commands.spawn().insert(KinematicBody {
                    pos: start_point,
                    mass: creation_data.mass,
                    radius: creation_data.radius,
                    vel: (start_point - mouse_state.prev_position) / 100.0,
                    ..Default::default()
                });

                *creation_state = CreationState::Initiated;
            } else {
                let current_mouse_position =
                    camera_res.camera.screen_to_world(mouse_position().into());
                let mouse_diff = mouse_state.prev_position - current_mouse_position;

                let mouse_moved = mouse_diff.length_squared() > 15.0;
                let preview_key_pressed =
                    is_key_pressed(KeyCode::P) || is_mouse_button_pressed(MouseButton::Right);

                if (!multi_preview.0 && mouse_moved) || (multi_preview.0 && preview_key_pressed) {
                    commands
                        .spawn()
                        .insert(KinematicBody {
                            pos: start_point,
                            vel: (start_point - mouse_state.prev_position) / 100.0,
                            mass: 1.0,
                            radius: creation_data.radius,
                            ..KinematicBody::default()
                        })
                        .insert(Color::new(0.5, 0.7, 1.0, 0.8))
                        .insert(Preview);

                    if !multi_preview.0 {
                        preview_query.iter().for_each(|entity| {
                            commands.entity(entity).despawn();
                        });
                    }
                }
            }
        }
    }
}
