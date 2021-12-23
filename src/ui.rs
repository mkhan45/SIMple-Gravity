pub mod body_creation;
pub mod input_state;
pub mod inspect;
pub mod top_panel;

use bevy_ecs::prelude::*;
use egui_macroquad::egui;

pub fn initialize_gui_sys(mut egui_ctx_res: ResMut<egui::CtxRef>) {
    egui_macroquad::ui(|egui_ctx| *egui_ctx_res = egui_ctx.clone());
}
