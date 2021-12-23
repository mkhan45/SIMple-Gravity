pub mod body_creation;
pub mod input_state;
pub mod inspect;
pub mod top_panel;

use bevy_ecs::prelude::*;
use egui_macroquad::egui;

pub fn initialize_gui_sys(mut egui_ctx_res: ResMut<egui::CtxRef>) {
    egui_macroquad::ui(|egui_ctx| {
        use egui::{FontDefinitions, TextStyle};
        let mut fonts = FontDefinitions::default();
        fonts.family_and_size.get_mut(&TextStyle::Button).unwrap().1 = 28.0;
        fonts.family_and_size.get_mut(&TextStyle::Body).unwrap().1 = 28.0;
        egui_ctx.set_fonts(fonts);

        *egui_ctx_res = egui_ctx.clone();
    });
}
