use bevy_ecs::prelude::*;
use egui_macroquad::egui::{self, RichText, TextStyle};
use egui_macroquad::macroquad::prelude::*;

use std::sync::{Arc, RwLock};

use crate::scripting::RhaiRes;

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
    egui_ctx: Res<egui::Context>,
    mut code_editor: ResMut<CodeEditor>,
    entities: Query<Entity>,
    mut commands: Commands,
    rhai: Res<RhaiRes>,
) {
    let mut shown = code_editor.shown;
    let mut ace_shown = shown;

    egui::Window::new("Scripting")
        .open(&mut shown)
        .resizable(true)
        .default_height(0.6 * screen_height())
        .default_width(0.5 * screen_width())
        .collapsible(false)
        .show(&egui_ctx, |ui| {
            // TODO: encode/decode all text so you cant escape it 💀
            let collapse_resp = 
                egui::CollapsingHeader::new("Editor").default_open(true).show(ui, |ui| {
                    ui.set_max_height(screen_height() * 0.6);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_min_width(screen_width() * 0.4);
                            ui.set_min_height(screen_height() * 0.5);
                            let padding = ui.style().spacing.window_margin.left_top();
                            let pos = ui.clip_rect().min + padding;
                            let y_offs = 32.0; // bc of the collapse title

                            let js_code = format!("set_editor_pos({}, {});", pos.x, pos.y + y_offs);
                            js_sys::eval(&js_code);
                        });
                        // egui::ScrollArea::vertical()
                        //     .min_scrolled_height(screen_height() * 0.5)
                        //     .show(ui, |ui| {
                        //     });

                        let mut code = code_editor.code.write().unwrap();
                        ui.vertical(|ui| {
                            ui.set_min_width(screen_width() * 0.1);
                            for (name, script) in crate::scripting::samples::SAMPLE_SCRIPTS {
                                if ui.button(name).clicked() {
                                    // *code = script.to_string();
                                    js_sys::eval(&format!("set_editor_code(`{}`)", script.to_string())).unwrap();
                                }
                            }
                        });
                    });
                });

            if let None = collapse_resp.body_response {
                ace_shown = false;
            }

            ui.add(egui::widgets::Separator::default().horizontal());
            ui.horizontal(|ui| {
                if ui.button("Run").clicked() {
                    code_editor.output = None;
                    code_editor.should_run = true;
                }
                if ui.button("Clear Scene & Run").clicked() {
                    code_editor.output = None;
                    entities.iter().for_each(|e| commands.entity(e).despawn());
                    rhai.existing_bodies.write().unwrap().clear();
                    rhai.newly_added_bodies.write().unwrap().clear();
                    code_editor.should_run = true;
                }

                if code_editor.should_run {
                    let mut code = code_editor.code.write().unwrap();
                    let code_str = js_sys::eval("get_editor_code()").unwrap();
                    *code = code_str.as_string().unwrap();
                }

                #[cfg(target_arch = "wasm32")]
                if ui.button("Download").clicked() {
                    let code = code_editor.code.read().unwrap();

                    let js_code = format!(
                        "
                                    const el = document.createElement('a');
                                    el.setAttribute('download', 'exported_script.rhai');

                                    const file_blob = new Blob([`{code}`], {{type: 'text/plain'}});
                                    el.setAttribute('href', URL.createObjectURL(file_blob));

                                    el.style.display = 'none';
                                    document.body.appendChild(el);

                                    el.click();

                                    document.body.removeChild(el);
                                "
                    );

                    let _ = js_sys::eval(&js_code);
                    // let js_dl_link = wasm_bindgen::JsValue::from_str(&dl_link);
                    // let log_args = js_sys::Array::of1(&js_dl_link);
                    // web_sys::console::log(&log_args);
                }
            });

            if let Some(output) = code_editor.output.clone() {
                let output = output.read().unwrap();
                let text =
                    RichText::new(format!("Output:\n{}", &output)).text_style(TextStyle::Monospace);

                ui.add_space(30.0);
                egui::ScrollArea::vertical().stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.label(text);
                    });
            }

            // egui::TopBottomPanel::bottom("Run")
            //     .resizable(false)
            //     .default_height(0.0)
            //     .show_inside(ui, |ui| {
            //     });

            // egui::CentralPanel::default().show_inside(ui, |ui| {
            // });
        });

    let js_code_shown = format!("set_editor_visibility({});", ace_shown);
    js_sys::eval(&js_code_shown);

    code_editor.shown = shown;
}
