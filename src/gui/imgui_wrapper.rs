use ggez::event::{KeyCode, KeyMods};
use ggez::{graphics, Context};

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

extern crate imgui;

#[allow(unused_imports)]
use imgui::StyleColor;
use imgui::*;
use imgui_gfx_renderer::*;

use crate::ecs::systems::graph_sys::GraphType;
use crate::gui::ui::*;
use crate::Vector;

use specs::prelude::*;

use std::collections::HashSet;
use std::time;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
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
    ToggleGraphs,
    SaveState,
    LoadState,
    DeleteAll,
    ToggleFollowBody,
    ToggleTrails,
    ToggleRelativeTrails,
}

#[derive(Debug, Clone)]
pub struct RenderData {
    pub create_mass: f32,
    pub create_rad: f32,
    pub mass: f32,
    pub rad: f32,
    pub dt: f32,
    pub num_iterations: usize,
    pub preview_iterations: usize,
    pub entity_selected: bool,
    pub save_filename: ImString,
    pub load_filename: ImString,
    pub trail_len: usize,
}

impl RenderData {
    pub fn new() -> Self {
        RenderData {
            create_mass: 0.01,
            create_rad: 1.0,
            mass: 0.01,
            rad: 1.0,
            dt: 1.0,
            num_iterations: 1,
            preview_iterations: 25,
            entity_selected: false,
            save_filename: ImString::new("save.ron"),
            load_filename: ImString::new("load.ron"),
            trail_len: 35,
        }
    }
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
    pub render_data: RenderData,
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

        {
            let io = imgui.io_mut();
            io[Key::Tab] = KeyCode::Tab as _;
            io[Key::LeftArrow] = KeyCode::Left as _;
            io[Key::RightArrow] = KeyCode::Right as _;
            io[Key::UpArrow] = KeyCode::Up as _;
            io[Key::DownArrow] = KeyCode::Down as _;
            io[Key::PageUp] = KeyCode::PageUp as _;
            io[Key::PageDown] = KeyCode::PageDown as _;
            io[Key::Home] = KeyCode::Home as _;
            io[Key::End] = KeyCode::End as _;
            io[Key::Insert] = KeyCode::Insert as _;
            io[Key::Delete] = KeyCode::Delete as _;
            io[Key::Backspace] = KeyCode::Back as _;
            io[Key::Space] = KeyCode::Space as _;
            io[Key::Enter] = KeyCode::Return as _;
            io[Key::Escape] = KeyCode::Escape as _;
            io[Key::KeyPadEnter] = KeyCode::NumpadEnter as _;
        }

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
            render_data: RenderData::new(),
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut Context,
        hidpi_factor: f32,
        items_hovered: &mut bool,
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
                            &mut self.sent_signals,
                            &mut self.render_data,
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
            make_menu_bar(&mut ui, &mut self.sent_signals, &mut self.render_data);
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
            self.remove_sidemenu();
        }
        if !self.graph {
            self.shown_menus.remove(&UiChoice::Graph);
        }
    }

    pub fn remove_sidemenu(&mut self) {
        self.shown_menus.retain(|menu| match menu {
            UiChoice::SideMenu(_) => false,
            _ => true,
        });
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
