use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

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
