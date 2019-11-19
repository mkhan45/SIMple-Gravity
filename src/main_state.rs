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
use crate::{
    imgui_wrapper::*, new_body, Body, Draw, Kinematics, Mass, Point, Position, Radius, Vector,
};

use std::cell::RefCell;
use std::convert::TryInto;

pub const DT: f32 = 1.0;

pub struct MainState {
    pub universe: Universe,
    pub main_world: World,
    pub imgui_wrapper: ImGuiWrapper,
    pub hidpi_factor: f32,
    pub resolution: Vector,
    pub selected_entity: Option<Entity>,
    pub mass: f32,
    pub rad: f32,
    pub dt: f32,
    pub num_iterations: i32,
    pub creating: bool,
    pub start_point: Option<Point>,
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
            mass: 0.1,
            rad: 1.0,
            num_iterations: 1,
            creating: false,
            start_point: None,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.imgui_wrapper
            .sent_signals
            .clone()
            .drain(..)
            .for_each(|signal| match signal {
                UiSignal::Create => self.creating = !self.creating,
            });
        self.imgui_wrapper.sent_signals.clear();

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

        let p = if let Some(start_pos) = self.start_point {
            start_pos
        } else {
            let mut mouse_pos = ggez::input::mouse::position(ctx);
            let coords = ggez::graphics::screen_coordinates(ctx);
            mouse_pos.x *= coords.w / self.resolution.x;
            mouse_pos.y *= coords.h / self.resolution.y;
            mouse_pos.into()
        };

        if self.creating {
            builder.circle(
                DrawMode::fill(),
                p,
                self.rad,
                0.05,
                Color::new(1.0, 1.0, 1.0, 0.5),
            );

            if let Some(p) = self.start_point {
                let mut mouse_pos = ggez::input::mouse::position(ctx);
                let coords = ggez::graphics::screen_coordinates(ctx);
                mouse_pos.x *= coords.w / self.resolution.x;
                mouse_pos.y *= coords.h / self.resolution.y;
                builder
                    .line(&[p, mouse_pos.into()], 0.5, graphics::WHITE)
                    .expect("not enough points in line");
            }
        }

        let mesh = builder.build(ctx).expect("error building mesh");

        // self.imgui_wrapper.shown_menus.push(UiChoice::DefaultUI);

        ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::new()).expect("error drawing mesh");
        let hidpi_factor = self.hidpi_factor;
        self.imgui_wrapper.render(
            ctx,
            hidpi_factor,
            &mut self.dt,
            &mut self.mass,
            &mut self.rad,
            &mut self.num_iterations,
            &mut self.creating,
        );

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
            MouseButton::Left => {
                if self.creating {
                    let mut p = Point::new(x, y);
                    let coords = ggez::graphics::screen_coordinates(ctx);
                    p.x *= coords.w / self.resolution.x;
                    p.y *= coords.h / self.resolution.y;
                    self.start_point = Some(p);
                }
            }
            _ => {}
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.selected_entity = None;
        self.imgui_wrapper.update_mouse_down((false, false, false));
        match button {
            MouseButton::Left => {
                if self.creating && !self.imgui_wrapper.sent_signals.contains(&UiSignal::Create) {
                    let mut p = Point::new(x, y);
                    let coords = ggez::graphics::screen_coordinates(ctx);
                    p.x *= coords.w / self.resolution.x;
                    p.y *= coords.h / self.resolution.y;

                    self.main_world.insert_from(
                        (),
                        vec![new_body(
                            self.start_point.unwrap(),
                            (self.start_point.unwrap() - p) * 0.10,
                            self.mass,
                            self.rad,
                        )],
                    );
                    self.start_point = None;
                }
            }
            _ => dbg!(),
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let aspect_ratio = height / width;
        graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0., 0., crate::SCREEN_X, crate::SCREEN_Y * aspect_ratio as f32),
        ).expect("error resizing");
        let resolution = Vector::new(width, height);
        self.imgui_wrapper.resolution = resolution;
        self.resolution = resolution;
    }
}
