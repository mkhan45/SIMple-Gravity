use bevy_ecs::prelude::*;
use egui::menu;
use egui_macroquad::egui;

use crate::{
    force_lines::DrawForceLines,
    physics::{Paused, DT, G},
    preview::MultiPreview,
    trails::{DrawTrails, RelativeTrails},
};

use super::{
    body_creation::{CreationData, CreationState},
    code_editor::CodeEditor,
};

pub fn top_panel_sys(
    egui_ctx: Res<egui::CtxRef>,
    mut creation_state: ResMut<CreationState>,
    mut creation_data: ResMut<CreationData>,
    mut paused: ResMut<Paused>,
    mut draw_force_lines: ResMut<DrawForceLines>,
    mut draw_trails: ResMut<DrawTrails>,
    mut relative_trails_body: ResMut<RelativeTrails>,
    mut code_editor: ResMut<CodeEditor>,
    mut multi_preview: ResMut<MultiPreview>,
    mut g: ResMut<G>,
    mut dt: ResMut<DT>,
    entities: Query<Entity>,
    mut commands: Commands,
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

            menu::menu(ui, "Options", |ui| {
                ui.checkbox(&mut draw_force_lines.0, "Draw Force Lines");
                ui.checkbox(&mut draw_trails.0, "Draw Trails");
                ui.checkbox(&mut multi_preview.0, "Multi Preview");
                ui.add(egui::Slider::new(&mut dt.0, 0.0..=10.0).text("Timestep"));
                ui.add(egui::Slider::new(&mut g.0, 0.0..=1000.0).text("G"));

                if ui.button("Stop Relative Trails").clicked() {
                    *relative_trails_body = RelativeTrails(None);
                }
            });

            if ui.button("Scripting").clicked() {
                code_editor.shown = !code_editor.shown;
            }

            if ui.button("Pause").clicked() {
                paused.0 = !paused.0;
            }

            if ui.button("Clear Scene").clicked() {
                for entity in entities.iter() {
                    commands.entity(entity).despawn();
                }
            }
        });
    });
}
