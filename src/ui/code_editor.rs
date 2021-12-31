use bevy_ecs::prelude::*;
use egui_macroquad::egui;
use egui_macroquad::macroquad::prelude::*;

use std::sync::{Arc, Mutex, RwLock};

pub struct CodeEditor {
    pub shown: bool,
    pub code: Arc<Mutex<String>>,
    pub should_run: bool,
    pub output: Option<Arc<RwLock<String>>>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            shown: false,
            code: Arc::new(Mutex::new("".to_string())),
            should_run: false,
            output: None,
        }
    }
}

pub fn code_editor_sys(
    egui_ctx: Res<egui::CtxRef>,
    mut code_editor: ResMut<CodeEditor>,
    entities: Query<Entity>,
    mut commands: Commands,
) {
    let mut shown = code_editor.shown;
    egui::Window::new("Scripting")
        .open(&mut shown)
        .resizable(true)
        .show(&egui_ctx, |ui| {
            ui.set_min_width(0.5 * screen_width());

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let mut code = code_editor.code.lock().unwrap();
                    ui.add(
                        egui::TextEdit::multiline(&mut *code)
                            .code_editor()
                            .desired_width(0.4 * screen_width())
                            .desired_rows(30),
                    );
                    std::mem::drop(code);

                    if ui.button("Run").clicked() {
                        code_editor.should_run = true;
                    }
                    if ui.button("Clear Scene & Run").clicked() {
                        entities.iter().for_each(|e| commands.entity(e).despawn());
                        code_editor.should_run = true;
                    }

                    if let Some(output) = code_editor.output.clone() {
                        let output = output.read().unwrap();
                        ui.add(
                            egui::Label::new(format!("Output:\n{}", &output))
                                .monospace()
                                .wrap(true),
                        );
                    }
                });

                ui.vertical(|ui| {
                    let mut code = code_editor.code.lock().unwrap();
                    for (name, script) in crate::scripting::samples::SAMPLE_SCRIPTS {
                        if ui.button(name).clicked() {
                            *code = script.to_string();
                        }
                    }
                });
            });
        });

    code_editor.shown = shown;
}
