use egui::menu;
use egui_macroquad::egui;

use bevy_ecs::prelude::*;

use super::body_creation::CreationData;

pub fn top_panel_sys(mut creation_data: ResMut<CreationData>) {
    egui_macroquad::ui(|egui_ctx| {
        use egui::{FontDefinitions, TextStyle};
        let mut fonts = FontDefinitions::default();
        fonts.family_and_size.get_mut(&TextStyle::Button).unwrap().1 = 24.0;
        fonts.family_and_size.get_mut(&TextStyle::Body).unwrap().1 = 28.0;
        egui_ctx.set_fonts(fonts);
        egui::TopBottomPanel::top("SIMple Gravity").show(egui_ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu(ui, "Create", |ui| {
                    ui.spacing_mut().slider_width = 500.0;
                    ui.add(
                        egui::Slider::new(&mut creation_data.mass, 0.0..=10_000.0)
                            .text("Mass")
                            .logarithmic(true),
                    );
                    ui.add(
                        egui::Slider::new(&mut creation_data.radius, 0.0..=1_000.0)
                            .text("Radius")
                            .logarithmic(true),
                    );
                })
            });
        });
    });
}
