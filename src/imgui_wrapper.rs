use ggez::{graphics, Context};

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

extern crate imgui;

use imgui::*;
use imgui_gfx_renderer::*;

use legion::prelude::Entity;

use std::time;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UiChoice {
    DefaultUI,
    SideMenu(Option<Entity>),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UiSignal {
}

pub struct ImGuiWrapper {
    pub imgui: imgui::Context,
    pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    pub last_frame: time::Instant,
    pub mouse_state: MouseState,
    pub shown_menus: Vec<UiChoice>,
    pub sent_signals: Vec<UiSignal>,
}

pub fn make_sidepanel(ui: &mut imgui::Ui, handle: &Option<Entity>) {
    // Window
    ui.window(im_str!("Hello world"))
        .position([300.0, 0.0], imgui::Condition::Always)
        .size([300.0, 600.0], imgui::Condition::Appearing)
        .collapsible(false)
        .build(|| {
            ui.text(im_str!("Hello world!"));
            ui.separator();

            if ui.small_button(im_str!("small button")) {
                println!("Small button clicked");
            }
        });
}

pub fn make_default_ui(ui: &mut imgui::Ui) {
    // Window
    ui.window(im_str!("Hello world"))
        .position([100.0, 100.0], imgui::Condition::Appearing)
        .build(|| {
            ui.text(im_str!("Hello world!"));
            ui.separator();

            if ui.small_button(im_str!("small button")) {
                println!("Small button clicked");
            }
        });
}

impl ImGuiWrapper {
    pub fn new(ctx: &mut ggez::Context, hidpi_factor: f32) -> Self {
        // Create the imgui object
        let mut imgui = imgui::Context::create();
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
            shown_menus: Vec::with_capacity(2),
            sent_signals: Vec::with_capacity(1),
        }
    }

    pub fn render(&mut self, ctx: &mut Context, hidpi_factor: f32) {
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
                match menu {
                    UiChoice::DefaultUI => make_default_ui(&mut ui),
                    UiChoice::SideMenu(entity) => make_sidepanel(&mut ui, entity),
                    _ => unimplemented!(),
                }
            }
        }

        // Render
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                &mut *factory,
                encoder,
                &mut RenderTargetView::new(render_target.clone()),
                draw_data,
            )
            .unwrap();
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
}
