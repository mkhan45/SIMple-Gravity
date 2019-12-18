use specs::prelude::*;

use ggez::{
    event,
    event::EventHandler,
    graphics,
    graphics::{Color, DrawMode},
    input,
    input::{
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton,
    },
    Context, GameResult,
};

use crate::resources::{
    CreateVec, DelSet, MainIterations, MousePos, Paused, PreviewIterations, Resolution, StartPoint,
    DT,
};
#[allow(unused_imports)]
use crate::{
    imgui_wrapper::*, new_body, new_preview, Body, Draw, Kinematics, Mass, Point, Position,
    Preview, PreviewBody, Radius, SpeedGraph, Trail, Vector,
};

static TRAIL_COLOR: graphics::Color = graphics::Color::new(0.2, 0.35, 1.0, 1.0);

use std::collections::HashSet;

const CAMERA_SPEED: f32 = 1.5;

pub fn create_body(world: &mut World, body: Body) -> Entity {
    world
        .create_entity()
        .with(body.0)
        .with(body.1)
        .with(body.2)
        .with(body.3)
        .with(body.4)
        .with(body.5)
        .build()
}
fn create_preview(world: &mut World, body: PreviewBody) -> Entity {
    world
        .create_entity()
        .with(body.0)
        .with(body.1)
        .with(body.2)
        .with(body.3)
        .with(body.4)
        .with(body.5)
        .build()
}

pub fn scale_pos(point: impl Into<Point>, coords: graphics::Rect, resolution: Vector) -> Point {
    let mut np: Point = point.into();
    np.x *= coords.w / resolution.x;
    np.y *= coords.h / resolution.y;
    np.x += coords.x;
    np.y += coords.y;
    np
}

pub struct MainState<'a, 'b> {
    pub world: World,
    pub main_dispatcher: Dispatcher<'a, 'b>,
    pub preview_dispatcher: Dispatcher<'a, 'b>,
    pub imgui_wrapper: ImGuiWrapper,
    pub hidpi_factor: f32,
    pub selected_entity: Option<Entity>,
    pub mass: f32,
    pub rad: f32,
    pub creating: bool,
    pub items_hovered: bool,
}

impl<'a, 'b> MainState<'a, 'b> {
    pub fn new(
        world: World,
        main_dispatcher: Dispatcher<'a, 'b>,
        preview_dispatcher: Dispatcher<'a, 'b>,
        imgui_wrapper: ImGuiWrapper,
        hidpi_factor: f32,
    ) -> Self {
        MainState {
            world,
            main_dispatcher,
            preview_dispatcher,
            imgui_wrapper,
            hidpi_factor,
            selected_entity: None,
            mass: 0.1,
            rad: 1.0,
            creating: false,
            items_hovered: false,
        }
    }
}

fn calc_offset(ctx: &Context) -> Vector {
    let mut offset: Vector = Vector::new(0.0, 0.0);

    if input::keyboard::is_key_pressed(ctx, KeyCode::Up)
        || input::keyboard::is_key_pressed(ctx, KeyCode::W)
    {
        offset.y -= CAMERA_SPEED;
    }
    if input::keyboard::is_key_pressed(ctx, KeyCode::Down)
        || input::keyboard::is_key_pressed(ctx, KeyCode::S)
    {
        offset.y += CAMERA_SPEED;
    }
    if input::keyboard::is_key_pressed(ctx, KeyCode::Left)
        || input::keyboard::is_key_pressed(ctx, KeyCode::A)
    {
        offset.x -= CAMERA_SPEED;
    }
    if input::keyboard::is_key_pressed(ctx, KeyCode::Right)
        || input::keyboard::is_key_pressed(ctx, KeyCode::D)
    {
        offset.x += CAMERA_SPEED;
    }

    offset
}

impl<'a, 'b> EventHandler for MainState<'a, 'b> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.world
            .insert(MousePos(input::mouse::position(ctx).into()));

        self.imgui_wrapper
            .sent_signals
            .clone()
            .drain(..)
            .for_each(|signal| match signal {
                UiSignal::Create => self.creating = !self.creating,
                UiSignal::Delete => {
                    if let Some(e) = self.selected_entity {
                        self.world
                            .delete_entity(e)
                            .expect("error deleting selected_entity");
                        self.selected_entity = None;
                    }
                }
                UiSignal::AddGraph(graph_type) => {
                    if let Some(e) = self.selected_entity {
                        match graph_type {
                            GraphType::Speed => {
                                let mut speed_graphs = self.world.write_storage::<SpeedGraph>();
                                speed_graphs
                                    .insert(e, SpeedGraph(Vec::with_capacity(750)))
                                    .expect("error adding graph");
                            }
                        }
                    }
                }
            });
        self.imgui_wrapper.sent_signals.clear();

        let offset = calc_offset(ctx);
        if offset != [0.0, 0.0].into() {
            let mut screen_coordinates = ggez::graphics::screen_coordinates(ctx);
            let zoom = screen_coordinates.w / crate::SCREEN_X;

            screen_coordinates.x += offset.x * zoom;
            screen_coordinates.y += offset.y * zoom;

            ggez::graphics::set_screen_coordinates(ctx, screen_coordinates).unwrap_or(());
        }

        if ggez::timer::ticks(ctx) % 60 == 0 {
            dbg!(ggez::timer::fps(ctx));
        }

        {
            let create_vec = self.world.fetch::<CreateVec>().0.clone();

            create_vec.iter().for_each(|body| {
                create_body(&mut self.world, body.clone());
            });
        }
        {
            let del_set = &self.world.fetch::<DelSet>().0.clone();

            del_set.iter().for_each(|e| {
                self.world
                    .delete_entity(*e)
                    .expect("error deleting collided entity");
            });
        }

        let preview_iterations = self.world.fetch::<PreviewIterations>().0;
        if !self.world.fetch::<Paused>().0 {
            let main_iterations = self.world.fetch::<MainIterations>().0;

            // do_physics(&mut self.world, ctx);
            (0..main_iterations).for_each(|_| {
                self.main_dispatcher.dispatch(&self.world);
            });

            (main_iterations..preview_iterations).for_each(|_| {
                self.preview_dispatcher.dispatch(&self.world);
            });
        } else {
            (0..preview_iterations).for_each(|_| {
                self.preview_dispatcher.dispatch(&self.world);
            });
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        let mut builder = graphics::MeshBuilder::new();

        {
            let draws = self.world.read_storage::<Draw>();
            let positions = self.world.read_storage::<Position>();
            let radii = self.world.read_storage::<Radius>();
            let previews = self.world.read_storage::<Preview>();
            let trails = self.world.read_storage::<Trail>();

            (&trails, &radii).join().for_each(|(trail, radius)| {
                let slices = trail.0.as_slices();
                if slices.0.len() >= 2 {
                    if let Err(e) = builder.line(slices.0, 0.25 * radius.0, TRAIL_COLOR) {
                        dbg!(e);
                    };
                }
                if slices.1.len() >= 2 {
                    if let Err(e) = builder.line(slices.1, 0.25 * radius.0, TRAIL_COLOR) {
                        dbg!(e);
                    };
                }
            });

            (&draws, &positions, &radii)
                .join()
                .for_each(|(color, pos, rad)| {
                    let point: ggez::mint::Point2<f32> = (*pos).into();
                    builder.circle(DrawMode::fill(), point, rad.0, 0.01, color.0);
                });

            (&previews, &positions, &radii)
                .join()
                .for_each(|(_, pos, rad)| {
                    let point: ggez::mint::Point2<f32> = (*pos).into();
                    let color = Color::new(0.1, 1.0, 0.2, 1.0);
                    builder.circle(DrawMode::fill(), point, rad.0, 0.05, color);
                });
        }

        let start_point = self.world.fetch::<StartPoint>().0;
        let resolution = self.world.fetch::<Resolution>().0;

        let p = if let Some(start_pos) = start_point {
            start_pos
        } else {
            let mouse_pos = ggez::input::mouse::position(ctx);
            let coords = ggez::graphics::screen_coordinates(ctx);
            scale_pos(mouse_pos, coords, resolution)
        };

        if self.creating {
            builder.circle(
                DrawMode::fill(),
                p,
                self.rad,
                0.05,
                Color::new(1.0, 1.0, 1.0, 0.5),
            );

            if let Some(p) = start_point {
                let mouse_pos = ggez::input::mouse::position(ctx);
                let coords = ggez::graphics::screen_coordinates(ctx);
                let scaled_pos = scale_pos(mouse_pos, coords, resolution);
                builder
                    .line(&[p, scaled_pos], 0.5, graphics::WHITE)
                    .expect("not enough points in line");
            }
        }

        let mesh = builder.build(ctx).expect("error building mesh");

        // self.imgui_wrapper.shown_menus.push(UiChoice::DefaultUI);

        ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::new()).expect("error drawing mesh");
        let hidpi_factor = self.hidpi_factor;

        let mut dt = if self.world.has_value::<DT>() {
            self.world.fetch::<DT>().0
        } else {
            self.world.insert(DT(1.0));
            1.0
        };

        let mut main_iter = self.world.fetch::<MainIterations>().0;
        let mut preview_iter = self.world.fetch::<PreviewIterations>().0;

        if let Some(e) = self.selected_entity {
            let (mut mass, mut rad) = {
                let masses = self.world.read_storage::<Mass>();
                let radii = self.world.read_storage::<Radius>();

                (masses.get(e).unwrap().0, radii.get(e).unwrap().0)
            };

            if self.world.is_alive(e) {
                self.imgui_wrapper.render(
                    ctx,
                    hidpi_factor,
                    &mut dt,
                    &mut mass,
                    &mut rad,
                    &mut main_iter,
                    &mut preview_iter,
                    &mut self.items_hovered,
                    true,
                );

                {
                    let mut masses_mut = self.world.write_storage::<Mass>();
                    let mut radii_mut = self.world.write_storage::<Radius>();

                    masses_mut.insert(e, Mass(mass)).unwrap_or(None);
                    radii_mut.insert(e, Radius(rad)).unwrap_or(None);
                }

                self.world.entities().entity(e.id());
            } else {
                self.selected_entity = None;
            }
        } else {
            self.imgui_wrapper.render(
                ctx,
                hidpi_factor,
                &mut dt,
                &mut self.mass,
                &mut self.rad,
                &mut main_iter,
                &mut preview_iter,
                &mut self.items_hovered,
                false,
            );
        }

        let mut graph_builder = graphics::MeshBuilder::new();
        crate::graph_sys::draw_graphs(&mut graph_builder, &self.world);
        let mesh = graph_builder.build(ctx).expect("error building mesh");
        ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::new())
            .expect("error drawing graph mesh");

        graphics::present(ctx)
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

        if !self.items_hovered {
            match button {
                MouseButton::Right => {
                    let resolution = self.world.fetch::<Resolution>().0;
                    self.imgui_wrapper.shown_menus.clear();
                    self.selected_entity = None;

                    let positions = self.world.read_storage::<Position>();
                    let radii = self.world.read_storage::<Radius>();
                    let entities = self.world.entities();

                    let coords = ggez::graphics::screen_coordinates(ctx);
                    let mouse_pos = scale_pos([x, y], coords, resolution);

                    for (e, pos, rad) in (&entities, &positions, &radii).join() {
                        if pos.dist(mouse_pos) <= rad.0 {
                            self.selected_entity = Some(e);
                            break;
                        }
                    }

                    self.imgui_wrapper
                        .shown_menus
                        .push(UiChoice::SideMenu(self.selected_entity));
                }
                MouseButton::Left => {
                    if self.creating {
                        let p = Point::new(x, y);
                        let coords = ggez::graphics::screen_coordinates(ctx);
                        let resolution = self.world.fetch::<Resolution>().0;
                        self.world
                            .insert(StartPoint(Some(scale_pos(p, coords, resolution))));

                        create_preview(&mut self.world, new_preview(p, [0.0, 0.0], self.rad));
                    }
                }
                _ => {}
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((false, false, false));
        let start_point = self.world.fetch::<StartPoint>().0;
        let resolution = self.world.fetch::<Resolution>().0;

        if let Some(start_point) = start_point {
            match button {
                MouseButton::Left => {
                    if self.creating && !self.imgui_wrapper.sent_signals.contains(&UiSignal::Create)
                    {
                        let mut p = Point::new(x, y);
                        let coords = ggez::graphics::screen_coordinates(ctx);
                        p = scale_pos(p, coords, resolution);

                        create_body(
                            &mut self.world,
                            new_body(start_point, (start_point - p) * 0.025, self.mass, self.rad),
                        );
                        self.world.insert(StartPoint(None));
                    }
                }
                _ => dbg!(),
            }
        }

        let mut delset: HashSet<Entity> = HashSet::new();

        {
            let previews = self.world.read_storage::<Preview>();
            let entities = self.world.entities();

            (&entities, &previews).join().for_each(|(entity, _)| {
                delset.insert(entity);
            });
        }

        delset.drain().for_each(|entity| {
            self.world
                .delete_entity(entity)
                .expect("error deleting collided entity");
        })
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);

        let mut delset: HashSet<Entity> = HashSet::new();

        {
            let previews = self.world.read_storage::<Preview>();
            let entities = self.world.entities();

            (&entities, &previews).join().for_each(|(entity, _)| {
                delset.insert(entity);
            });
        }

        delset.drain().for_each(|entity| {
            self.world
                .delete_entity(entity)
                .expect("error deleting collided entity");
        });

        let mut coords = ggez::graphics::screen_coordinates(ctx);

        let start_point = self.world.fetch::<StartPoint>().0;
        if let Some(sp) = start_point {
            let resolution = self.world.fetch::<Resolution>().0;
            let p = scale_pos([x, y], coords, resolution);

            create_preview(&mut self.world, new_preview(sp, (sp - p) * 0.025, self.rad));
        }

        if input::mouse::button_pressed(ctx, input::mouse::MouseButton::Middle) {
            let mut offset = Vector::new(dx, dy);
            let resolution = self.world.fetch::<Resolution>().0;
            offset.x *= coords.w / resolution.x;
            offset.y *= coords.h / resolution.y;
            coords.x -= offset.x;
            coords.y -= offset.y;
            graphics::set_screen_coordinates(ctx, coords).expect("error moving my mclick");
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        let mouse_pos = input::mouse::position(ctx);
        let mut offset = graphics::screen_coordinates(ctx);

        let prev_zoom = offset.w / crate::SCREEN_X;
        let zoom = prev_zoom * (1.0 - (y * 0.05));

        let resolution = self.world.fetch::<Resolution>().0;

        let focus: Vector = Vector::new(mouse_pos.x, mouse_pos.y);

        let mut scaled_focus1: Vector = focus;
        scaled_focus1.x *= offset.w / resolution.x;
        scaled_focus1.y *= offset.h / resolution.y;

        offset.w = zoom * crate::SCREEN_X;
        offset.h = zoom * crate::SCREEN_Y / (resolution.x / resolution.y);

        let mut scaled_focus2: Vector = focus;
        scaled_focus2.x *= offset.w / resolution.x;
        scaled_focus2.y *= offset.h / resolution.y;

        let delta_focus = scaled_focus2 - scaled_focus1;

        offset.x -= delta_focus.x;
        offset.y -= delta_focus.y;

        graphics::set_screen_coordinates(ctx, offset).unwrap_or(());
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        #[allow(clippy::single_match)]
        match keycode {
            KeyCode::Space => self.world.get_mut::<Paused>().unwrap().invert(),
            KeyCode::Escape => self.imgui_wrapper.shown_menus.clear(),
            _ => {}
        };
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let aspect_ratio = height / width;
        graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(
                0.,
                0.,
                crate::SCREEN_X,
                crate::SCREEN_Y * aspect_ratio as f32,
            ),
        )
        .expect("error resizing");
        let resolution = Vector::new(width, height);
        self.imgui_wrapper.resolution = resolution;
        self.world.insert(Resolution(resolution));
    }
}
