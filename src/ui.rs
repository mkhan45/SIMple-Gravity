pub mod body_creation;
pub mod code_editor;
pub mod input_state;
pub mod inspect;
pub mod top_panel;

use bevy_ecs::prelude::*;
use egui_macroquad::egui::CtxRef;

use crate::physics::Paused;

pub fn pause_unpause_sys(mut paused: ResMut<Paused>, egui_ctx: Res<CtxRef>) {
    use egui_macroquad::macroquad::prelude::*;

    if is_key_pressed(KeyCode::Space) && !egui_ctx.is_pointer_over_area() {
        paused.0 = !paused.0;
    }
}
