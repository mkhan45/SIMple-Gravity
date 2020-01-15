use ggez::event::{KeyCode, KeyMods};
use ggez::{graphics, Context};

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

extern crate imgui;

#[allow(unused_imports)]
use imgui::StyleColor;
use imgui::*;
use imgui_gfx_renderer::*;

use crate::Vector;

use specs::prelude::*;

use std::time;

use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GraphType {
    Speed,
    XVel,
    YVel,
    Accel,
}

#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
#[allow(dead_code)]
pub enum UiChoice {
    DefaultUI,
    SideMenu(Option<Entity>),
    Graph,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UiSignal {
    Create,
    Delete,
    AddGraph(GraphType),
    RemoveGraphs,
    SaveState,
}

pub struct ImGuiWrapper {
    pub imgui: imgui::Context,
    pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    pub last_frame: time::Instant,
    pub mouse_state: MouseState,
    pub shown_menus: HashSet<UiChoice>,
    pub sent_signals: Vec<UiSignal>,
    pub resolution: Vector,
    pub sidemenu: bool,
    pub graph: bool,
}

pub fn make_sidepanel(
    ui: &mut imgui::Ui,
    resolution: Vector,
    open_bool: &mut bool,
    mass: &mut f32,
    rad: &mut f32,
    dt: &mut f32,
    num_iterations: &mut usize,
    preview_iterations: &mut usize,
    signals: &mut Vec<UiSignal>,
    selected_entity: bool,
) {
    // Window
    let win = imgui::Window::new(im_str!("Menu"))
        .position([0.0, 0.0], imgui::Condition::Always)
        .opened(open_bool)
        .size(
            [resolution.x * 0.35, resolution.y],
            imgui::Condition::Appearing,
        )
        .collapsible(false)
        .movable(false)
        .size_constraints(
            [resolution.x * 0.1, resolution.y],
            [resolution.x * 0.6, resolution.y],
        );
    win.build(ui, || {
        if selected_entity {
            ui.text(im_str!("Edit Object"));
        } else {
            ui.text(im_str!("New Object"));
        }

        let mass_speed = (*mass * 0.0015).max(0.01);
        let rad_speed = (*rad * 0.0015).max(0.01);
        ui.drag_float(im_str!("Mass"), mass)
            .speed(mass_speed)
            .build();
        ui.drag_float(im_str!("Radius"), rad)
            .speed(rad_speed)
            .build();
        if ui.small_button(im_str!("Toggle Create Body")) {
            signals.push(UiSignal::Create);
        }

        if selected_entity {
            if ui.small_button(im_str!("Graph Speed")) {
                signals.push(UiSignal::AddGraph(GraphType::Speed));
            }

            if ui.small_button(im_str!("Graph X Velocity")) {
                signals.push(UiSignal::AddGraph(GraphType::XVel));
            }

            if ui.small_button(im_str!("Graph Y Velocity")) {
                signals.push(UiSignal::AddGraph(GraphType::YVel));
            }

            if ui.small_button(im_str!("Delete Body")) {
                signals.push(UiSignal::Delete);
            }
        }
        ui.separator();
        ui.text(im_str!("DT"));
        ui.drag_float(im_str!(""), dt).speed(0.01).build();

        ui.text(im_str!("Iteration Count"));
        let mut num_iterations_i32: i32 = *num_iterations as i32;
        ui.drag_int(im_str!(" "), &mut num_iterations_i32)
            .min(0)
            .speed(0.05 * (*num_iterations as f32).powf(1.0 / 3.0))
            .max(1000)
            .build();

        ui.text(im_str!("Preview Iteration Count"));
        let mut preview_iterations_i32: i32 = *preview_iterations as i32;
        ui.drag_int(im_str!("  "), &mut preview_iterations_i32)
            .min(0)
            .speed(0.05 * (*preview_iterations as f32).powf(1.0 / 3.0))
            .max(1000)
            .build();

        *num_iterations = num_iterations_i32 as usize;
        *preview_iterations = preview_iterations_i32 as usize;

        if ui.small_button(im_str!("Remove Graphs")) {
            signals.push(UiSignal::RemoveGraphs);
        }
        if ui.small_button(im_str!("Save the Universe")) {
            signals.push(UiSignal::SaveState);
        }
    });
}

pub fn make_graph_ui(
    ui: &mut imgui::Ui,
    resolution: Vector,
    open_bool: &mut bool,
    data: &[f32],
    graph_type: GraphType,
) {
    // Window

    let graph_name = match graph_type {
        GraphType::Speed => im_str!("Speed"),
        GraphType::XVel => im_str!("X Velocity"),
        GraphType::YVel => im_str!("Y Velocity"),
        _ => panic!("{:#?} is not an imgui graph", graph_type),
    };

    imgui::Window::new(im_str!("Graphs"))
        .position([resolution.x * 0.6, 0.0], imgui::Condition::Appearing)
        .size(
            [resolution.x * 0.4, resolution.y * 0.4],
            imgui::Condition::Appearing,
        )
        .opened(open_bool)
        .build(ui, || {
            ui.plot_lines(graph_name, data)
                .graph_size([resolution.x * 0.3, resolution.y * 0.3])
                .build();
        });
}

pub fn make_default_ui(ui: &mut imgui::Ui) {
    // Window
    imgui::Window::new(im_str!("Hello world"))
        .position([100.0, 100.0], imgui::Condition::Appearing)
        .build(ui, || {
            ui.text(im_str!("Hello world!"));
            ui.separator();

            if ui.small_button(im_str!("small button")) {
                println!("Small button clicked");
            }
        });
}

impl ImGuiWrapper {
    pub fn new(ctx: &mut ggez::Context, hidpi_factor: f32, resolution: Vector) -> Self {
        // Create the imgui objectdt: f32,
        let mut imgui = imgui::Context::create();
        let style = imgui.style_mut();
        style.window_rounding = 0.0;
        style.child_rounding = 0.0;
        style.popup_rounding = 0.0;
        style.frame_rounding = 0.0;
        style.tab_rounding = 0.0;
        style.grab_rounding = 0.0;
        let (factory, gfx_device, _, _, _) = graphics::gfx_objects(ctx);

        // Shaders
        let shaders = {
            let version = gfx_device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: 26.0 * hidpi_factor,
                ..FontConfig::default()
            }),
        }]);
        imgui.io_mut().font_global_scale = 1.0 / hidpi_factor;

        // Renderer
        let renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

        // Create instace
        Self {
            imgui,
            renderer,
            last_frame: time::Instant::now(),
            mouse_state: MouseState::default(),
            shown_menus: HashSet::with_capacity(2),
            sent_signals: Vec::with_capacity(1),
            resolution,
            sidemenu: false,
            graph: false,
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut Context,
        hidpi_factor: f32,
        dt: &mut f32,
        mass: &mut f32,
        rad: &mut f32,
        num_iterations: &mut usize,
        preview_iterations: &mut usize,
        items_hovered: &mut bool,
        selected_entity: bool,
        graph_data: Vec<(GraphType, &[f32])>,
    ) {
        // Update mouse
        self.update_mouse();

        // Create new frame
        let now = time::Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        self.imgui.io_mut().display_size = [draw_width, draw_height];
        self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        self.imgui.io_mut().delta_time = delta_s;

        let mut ui = self.imgui.frame();

        {
            for menu in self.shown_menus.clone().iter() {
                #[allow(unreachable_patterns)]
                match menu {
                    UiChoice::DefaultUI => make_default_ui(&mut ui),
                    UiChoice::SideMenu(_entity) => {
                        self.sidemenu = true;
                        make_sidepanel(
                            &mut ui,
                            self.resolution,
                            &mut self.sidemenu,
                            mass,
                            rad,
                            dt,
                            num_iterations,
                            preview_iterations,
                            &mut self.sent_signals,
                            selected_entity,
                        );
                    }
                    UiChoice::Graph => {
                        self.graph = true;
                        for (graph_type, data) in graph_data.iter() {
                            make_graph_ui(
                                &mut ui,
                                self.resolution,
                                &mut self.graph,
                                data,
                                *graph_type,
                            );
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
        *items_hovered = ui.is_any_item_hovered();

        // Render
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                &mut *factory,
                encoder,
                &mut RenderTargetView::new(render_target),
                draw_data,
            )
            .unwrap();

        if !self.sidemenu {
            self.shown_menus = self
                .shown_menus
                .iter()
                .filter(|menu| match menu {
                    UiChoice::SideMenu(_) => false,
                    _ => true,
                })
                .cloned()
                .collect();
        }
        if !self.graph {
            self.shown_menus.remove(&UiChoice::Graph);
        }
    }

    fn update_mouse(&mut self) {
        self.imgui.io_mut().mouse_pos =
            [self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32];

        self.imgui.io_mut().mouse_down = [
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ];

        self.imgui.io_mut().mouse_wheel = self.mouse_state.wheel;
        self.mouse_state.wheel = 0.0;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_state.pos = (x as i32, y as i32);
    }

    pub fn update_mouse_down(&mut self, pressed: (bool, bool, bool)) {
        self.mouse_state.pressed = pressed;
    }

    pub fn update_key_up(&mut self, key: KeyCode, mods: KeyMods) {
        if mods.contains(KeyMods::SHIFT) {
            self.imgui.io_mut().key_shift = false;
        }
        if mods.contains(KeyMods::CTRL) {
            self.imgui.io_mut().key_ctrl = false;
        }
        if mods.contains(KeyMods::ALT) {
            self.imgui.io_mut().key_alt = false;
        }

        self.imgui.io_mut().keys_down[key as usize] = false;
    }
    pub fn update_key_down(&mut self, key: KeyCode, mods: KeyMods) {
        if mods.contains(KeyMods::SHIFT) {
            self.imgui.io_mut().key_shift = true;
        }
        if mods.contains(KeyMods::CTRL) {
            self.imgui.io_mut().key_ctrl = true;
        }
        if mods.contains(KeyMods::ALT) {
            self.imgui.io_mut().key_alt = true;
        }

        self.imgui.io_mut().keys_down[key as usize] = true;
    }

    pub fn update_text(&mut self, val: char) {
        self.imgui.io_mut().add_input_character(val);
    }
}
