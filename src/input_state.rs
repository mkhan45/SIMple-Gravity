use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::{camera::CameraRes, physics::KinematicBody};

#[derive(Default)]
pub struct MouseState {
    pub prev_position: Vec2,
}

pub fn update_mouse_input_sys(mut mouse_state: ResMut<MouseState>, camera_res: Res<CameraRes>) {
    mouse_state.prev_position = camera_res.camera.screen_to_world(mouse_position().into());
}

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
            radius: 150.0,
            mass: 1.0,
        }
    }
}

pub fn create_body_sys(
    mut creation_state: ResMut<CreationState>,
    creation_data: Res<CreationData>,
    mouse_state: Res<MouseState>,
    mut commands: Commands,
) {
    match *creation_state {
        CreationState::Unstarted => {
            if is_key_pressed(KeyCode::C) {
                *creation_state = CreationState::Initiated;
            }
        }
        CreationState::Initiated => {
            if is_mouse_button_pressed(MouseButton::Left) {
                *creation_state = CreationState::Clicked {
                    start_point: mouse_state.prev_position,
                }
            }
        }
        CreationState::Clicked { start_point: _ } => {
            commands.spawn().insert(KinematicBody {
                pos: mouse_state.prev_position,
                mass: creation_data.mass,
                radius: creation_data.radius,
                ..Default::default()
            });

            *creation_state = CreationState::Initiated;
        }
    }
}
