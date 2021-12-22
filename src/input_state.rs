use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::camera::CameraRes;

#[derive(Default)]
pub struct MouseState {
    pub prev_position: Vec2,
}

pub fn update_mouse_input_sys(mut mouse_state: ResMut<MouseState>, camera_res: Res<CameraRes>) {
    mouse_state.prev_position = camera_res.camera.screen_to_world(mouse_position().into());
}
