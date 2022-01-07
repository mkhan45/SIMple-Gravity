use bevy_ecs::prelude::*;
use egui_macroquad::egui;
use egui_macroquad::macroquad::prelude::*;

use std::sync::{Arc, RwLock};

pub struct CodeEditor {
    pub shown: bool,
    pub code: Arc<RwLock<String>>,
    pub should_run: bool,
    pub output: Option<Arc<RwLock<String>>>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            shown: false,
            code: Arc::new(RwLock::new("".to_string())),
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
        .default_height(0.6 * screen_height())
        .default_width(0.5 * screen_width())
        .show(&egui_ctx, |ui| {
            egui::TopBottomPanel::bottom("Run")
                .resizable(true)
                .default_height(0.0)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            code_editor.should_run = true;
                        }
                        if ui.button("Clear Scene & Run").clicked() {
                            entities.iter().for_each(|e| commands.entity(e).despawn());
                            code_editor.should_run = true;
                        }

                        #[cfg(target_arch = "wasm32")]
                        if ui.button("Download").clicked() {
                            let code = code_editor.code.read().unwrap();

                                let js_code = format!("
                                    const el = document.createElement('a');
                                    el.setAttribute('download', 'exported_script.rhai');

                                    const file_blob = new Blob([`{code}`], {{type: 'text/plain'}});
                                    el.setAttribute('href', URL.createObjectURL(file_blob));

                                    el.style.display = 'none';
                                    document.body.appendChild(el);

                                    el.click();

                                    document.body.removeChild(el);
                                ");

                                let _ = js_sys::eval(&js_code);
                                // let js_dl_link = wasm_bindgen::JsValue::from_str(&dl_link);
                                // let log_args = js_sys::Array::of1(&js_dl_link);
                                // web_sys::console::log(&log_args);
                        }
                    });

                    if let Some(output) = code_editor.output.clone() {
                        let output = output.read().unwrap();
                        ui.add(
                            egui::Label::new(format!("Output:\n{}", &output))
                                .monospace()
                                .wrap(true),
                        );
                    }
                });

            egui::SidePanel::right("Samples")
                .resizable(true)
                .show_inside(ui, |ui| {
                    let mut code = code_editor.code.write().unwrap();
                    for (name, script) in crate::scripting::samples::SAMPLE_SCRIPTS {
                        if ui.button(name).clicked() {
                            *code = script.to_string();
                        }
                    }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut code = code_editor.code.write().unwrap();
                    ui.add(
                        egui::TextEdit::multiline(&mut *code)
                            .code_editor()
                            .desired_width(0.4 * screen_width())
                            .desired_rows(23),
                    );
                });
            });
        });

    code_editor.shown = shown;
}
