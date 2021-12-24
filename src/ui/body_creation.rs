use bevy_ecs::prelude::*;
use egui_macroquad::{macroquad, egui::CtxRef};
use macroquad::prelude::*;

use super::input_state::MouseState;
use crate::physics::KinematicBody;

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

            if is_mouse_button_pressed(MouseButton::Left) && !egui_ctx.input().pointer.has_pointer() {
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
            if is_mouse_button_released(MouseButton::Left) {
                commands.spawn().insert(KinematicBody {
                    pos: start_point,
                    mass: creation_data.mass,
                    radius: creation_data.radius,
                    vel: (start_point - mouse_state.prev_position) / 100.0,
                    ..Default::default()
                });

                *creation_state = CreationState::Initiated;
            }
        }
    }
}
