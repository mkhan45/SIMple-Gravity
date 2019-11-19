use legion::prelude::*;

use ggez::{
    event,
    event::EventHandler,
    graphics,
    graphics::{Color, DrawMode, DrawParam},
    input,
    input::{
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton,
    },
    Context, GameResult,
};

use crate::physics::{apply_gravity, calc_collisions, integrate_kinematics, integrate_positions};
use crate::{imgui_wrapper::*, Draw, Kinematics, Mass, Position, Radius, Vector};

use std::convert::TryInto;

pub const DT: f32 = 1.0;

pub struct MainState {
    pub universe: Universe,
    pub main_world: World,
    pub imgui_wrapper: ImGuiWrapper,
    pub hidpi_factor: f32,
    pub resolution: Vector,
    pub selected_entity: Option<Entity>,
    pub dt: f32,
    pub num_iterations: usize,
}

impl MainState {
    pub fn new(
        universe: Universe,
        main_world: World,
        imgui_wrapper: ImGuiWrapper,
        hidpi_factor: f32,
        resolution: Vector,
    ) -> Self {
        MainState {
            universe,
            main_world,
            imgui_wrapper,
            hidpi_factor,
            resolution,
            selected_entity: None,
            dt: DT,
            num_iterations: 1,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = self.imgui_wrapper.dt;
        match self.imgui_wrapper.num_iterations.try_into() {
            Ok(num) => self.num_iterations = num,
            Err(_) => self.imgui_wrapper.num_iterations = 1,
        }
        if ggez::timer::ticks(ctx) % 60 == 0 {
            dbg!(ggez::timer::fps(ctx));
        }
        for _ in 0..self.num_iterations {
            calc_collisions(&mut self.main_world);
            integrate_positions(&self.main_world, self.dt);
            apply_gravity(&self.main_world);
            integrate_kinematics(&self.main_world, self.dt);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        let mut builder = graphics::MeshBuilder::new();

        let draw_query = <(Read<Draw>, Read<Position>, Read<Radius>)>::query();

        draw_query
            .iter(&self.main_world)
            .for_each(|(color, pos, rad)| {
                let point: ggez::mint::Point2<f32> = (*pos).into();
                builder.circle(DrawMode::fill(), point, rad.0, 0.05, color.0);
            });

        let mesh = builder.build(ctx).expect("error building mesh");

        // self.imgui_wrapper.shown_menus.push(UiChoice::DefaultUI);

        ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::new()).expect("error drawing mesh");
        self.imgui_wrapper.render(ctx, self.hidpi_factor);

        ggez::graphics::present(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((
            button == MouseButton::Left,
            button == MouseButton::Right,
            button == MouseButton::Middle,
        ));

        // self.imgui_wrapper.shown_menus.clear();
        // self.imgui_wrapper.shown_menus.push(UiChoice::DefaultUI);

        match button {
            MouseButton::Right => {
                self.imgui_wrapper.shown_menus.clear();
                let clicked_query = <(Read<Position>, Read<Radius>)>::query();
                let mut entity: Option<Entity> = None;

                for (e, (pos, rad)) in clicked_query.iter_entities(&self.main_world) {
                    if pos.dist([x, y].into()) <= rad.0 {
                        entity = Some(e);
                        break;
                    }
                }

                self.imgui_wrapper
                    .shown_menus
                    .push(UiChoice::SideMenu(entity));
            }
            _ => {}
        }
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.selected_entity = None;
        self.imgui_wrapper.update_mouse_down((false, false, false));
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
    }
}
