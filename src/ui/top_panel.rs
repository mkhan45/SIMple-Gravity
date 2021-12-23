use bevy_ecs::prelude::*;
use egui::menu;
use egui_macroquad::egui;

use crate::physics::Paused;

use super::body_creation::{CreationData, CreationState};

pub fn top_panel_sys(
    egui_ctx: Res<egui::CtxRef>,
    mut creation_state: ResMut<CreationState>,
    mut creation_data: ResMut<CreationData>,
    mut paused: ResMut<Paused>,
) {
    egui::TopBottomPanel::top("SIMple Gravity").show(&egui_ctx, |ui| {
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

                if ui.button("Create").clicked() {
                    *creation_state = CreationState::Initiated;
                }
            });

            if ui.button("Pause").clicked() {
                paused.0 = !paused.0;
            }
        });
    });
}
