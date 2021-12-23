use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::input_state::MouseState;

const SCREEN_WIDTH: f32 = 10_000.0;
const SCREEN_HEIGHT: f32 = 10_000.0;

pub struct CameraRes {
    pub camera: Camera2D,
    pub screen_size: Vec2,
}

impl CameraRes {
    pub fn contains_point(&self, point: &Vec2) -> bool {
        let camera_left = self.camera.target.x - self.screen_size.x / 2.0;
        let camera_right = self.camera.target.x + self.screen_size.x / 2.0;
        let camera_top = self.camera.target.y - self.screen_size.y / 2.0;
        let camera_bottom = self.camera.target.y + self.screen_size.y / 2.0;

        (camera_left..camera_right).contains(&point.x)
            && (camera_top..camera_bottom).contains(&point.y)
    }
}

impl Default for CameraRes {
    fn default() -> Self {
        let display_rect = Rect::new(
            -SCREEN_WIDTH / 2.0,
            -SCREEN_HEIGHT / 2.0,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        );

        CameraRes {
            camera: Camera2D::from_display_rect(display_rect),
            screen_size: Vec2::new(screen_width(), screen_height()),
        }
    }
}

pub fn update_camera_sys(mut camera_res: ResMut<CameraRes>) {
    let screen_height_change = screen_height() / camera_res.screen_size.y;
    let aspect_ratio = camera_res.screen_size.x / camera_res.screen_size.y;

    camera_res.screen_size.x = screen_width();
    camera_res.screen_size.y = screen_height();

    camera_res.camera.zoom.y /= screen_height_change;
    camera_res.camera.zoom.x = camera_res.camera.zoom.y / aspect_ratio;

    camera_res.camera.zoom.x = camera_res.camera.zoom.x.abs();
    camera_res.camera.zoom.y = camera_res.camera.zoom.y.abs();

    set_camera(&camera_res.camera);
}

pub fn camera_transform_sys(mut camera_res: ResMut<CameraRes>, mouse_state: Res<MouseState>) {
    let mouse_screen_pos: Vec2 = mouse_position().into();
    let current_mouse_pos = camera_res.camera.screen_to_world(mouse_screen_pos);

    // panning via middle mouse
    if is_mouse_button_down(MouseButton::Middle) {
        let offset = current_mouse_pos - mouse_state.prev_position;
        camera_res.camera.target -= offset;
    }

    // zooming
    let y_scroll = mouse_wheel().1;

    if y_scroll != 0.0 {
        let scale_fac = 1.0 + y_scroll.signum() * 0.1;

        camera_res.screen_size *= scale_fac;
        camera_res.camera.zoom *= scale_fac;

        let old_world_mouse_pos = current_mouse_pos;
        let new_world_mouse_pos = camera_res.camera.screen_to_world(mouse_screen_pos);
        let mouse_delta = old_world_mouse_pos - new_world_mouse_pos;

        camera_res.camera.target += mouse_delta * 2.0;
    }
}
