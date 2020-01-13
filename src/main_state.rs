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

#[allow(unused_imports)]
use crate::ecs::components::{
    Draw, Kinematics, Mass, Position, Preview, Radius, SpeedGraph, Trail, XVelGraph,
};
use crate::ecs::entities::{create_body, create_preview, new_body, new_preview};
use crate::ecs::resources::{
    MainIterations, MousePos, NewPreview, Paused, PreviewIterations, Resolution, StartPoint, DT,
};
use crate::imgui_wrapper::*;
#[allow(unused_imports)]
use crate::{Point, Vector, SCREEN_X, SCREEN_Y};

static TRAIL_COLOR: graphics::Color = graphics::Color::new(0.2, 0.35, 1.0, 1.0);

use std::collections::HashSet;

const CAMERA_SPEED: f32 = 1.5;

pub fn scale_pos(point: impl Into<Point>, coords: graphics::Rect, resolution: Vector) -> Point {
    let mut np: Point = point.into();
    np.x *= coords.w / resolution.x;
    np.y *= coords.h / resolution.y;
    np.x += coords.x;
    np.y += coords.y;
    np
}

//TODO: move mass, rad, selected_entity, hidpi_factor, and creating to specs resources
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

        // process GUI events
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
                                if let None = speed_graphs.get(e) {
                                    speed_graphs
                                        .insert(e, SpeedGraph::new())
                                        .expect("error adding graph");
                                }
                            }
                            GraphType::XVel => {
                                let mut xvel_graphs = self.world.write_storage::<XVelGraph>();
                                if let None = xvel_graphs.get(e) {
                                    xvel_graphs
                                        .insert(e, XVelGraph::new())
                                        .expect("error adding graph");
                                }
                            }
                        }
                        if !self.imgui_wrapper.shown_menus.contains(&UiChoice::Graph) {
                            self.imgui_wrapper.shown_menus.insert(UiChoice::Graph);
                        }
                    }
                }
            });
        self.imgui_wrapper.sent_signals.clear();

        // unselect selected entity if it collided
        if let Some(e) = self.selected_entity {
            if !self.world.is_alive(e) {
                self.selected_entity = None;
            }
        }

        // if preview collided, delete it and make a new one
        if self.world.fetch::<NewPreview>().0 {
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
                    .expect("error deleting collided preview");
            });

            let coords = ggez::graphics::screen_coordinates(ctx);

            let start_point = self.world.fetch::<StartPoint>().0;
            if let Some(sp) = start_point {
                let resolution = self.world.fetch::<Resolution>().0;
                let mouse_pos = self.world.fetch::<MousePos>().0;
                let p = scale_pos([mouse_pos.x, mouse_pos.y], coords, resolution);

                create_preview(&mut self.world, new_preview(sp, (sp - p) * 0.025, self.rad));
            }

            self.world.insert(NewPreview(false));
        }

        // apply camera movement
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

        // run physics systems
        let preview_iterations = self.world.fetch::<PreviewIterations>().0;
        if !self.world.fetch::<Paused>().0 {
            let main_iterations = self.world.fetch::<MainIterations>().0;

            // do_physics(&mut self.world, ctx);
            (0..main_iterations).for_each(|_| {
                self.main_dispatcher.dispatch(&self.world);
            });
            self.world.maintain();
        }
        if let Some(e) = self.selected_entity {
            if !self.world.is_alive(e) {
                self.selected_entity = None;
            }
        }

        (0..preview_iterations).for_each(|_| {
            self.preview_dispatcher.dispatch(&self.world);
        });

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

            // draw trails
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

            // this is kind of inelegant but previews don't have the Draw component and color is
            // hardcoded
            // TODO?
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

        // draw new body preview and line
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
        ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::new()).expect("error drawing mesh");

        // Draw GUI and process sliders
        let hidpi_factor = self.hidpi_factor;

        let mut dt = self.world.fetch::<DT>().0;

        let mut main_iter = self.world.fetch::<MainIterations>().0;
        let mut preview_iter = self.world.fetch::<PreviewIterations>().0;

        let mut graph_data: Vec<(GraphType, &[f32])> = Vec::new();
        let speed_graphs = self.world.read_storage::<SpeedGraph>();
        let xvel_graphs = self.world.read_storage::<XVelGraph>();

        speed_graphs.join().for_each(|data| {
            graph_data.push((GraphType::Speed, &data.data[..]));
        });
        xvel_graphs.join().for_each(|data| {
            graph_data.push((GraphType::XVel, &data.data[..]));
        });

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
                    graph_data,
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
                graph_data,
            );
        }

        std::mem::drop(speed_graphs);
        std::mem::drop(xvel_graphs);

        self.world.insert(MainIterations(main_iter));
        self.world.insert(PreviewIterations(preview_iter));
        self.world.insert(DT(dt));

        let coords = ggez::graphics::screen_coordinates(ctx);
        let resolution = self.world.fetch::<Resolution>().0;

        let scale = [
            (coords.w / resolution.x) * 300.,
            coords.h / resolution.y * 300.,
        ];

        ggez::graphics::draw(
            ctx,
            &mesh,
            graphics::DrawParam::new().scale(Vector::new(scale[0], scale[1])), // .dest(Point::new(pos[0], pos[1]))
                                                                               // .offset(Point::new(-pos[0], -pos[1]))
        )
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
                    // delete clicked body
                    self.imgui_wrapper.shown_menus = self
                        .imgui_wrapper
                        .shown_menus
                        .iter()
                        .filter(|menu| match menu {
                            UiChoice::SideMenu(_) => false,
                            _ => true,
                        })
                        .cloned()
                        .collect();

                    let resolution = self.world.fetch::<Resolution>().0;
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
                        .insert(UiChoice::SideMenu(self.selected_entity));
                }
                MouseButton::Left => {
                    // set up for creating new body
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
                // create new body
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

        // delete preview
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

        // delete old preview create a new one
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
        // zoom
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
        match keycode {
            KeyCode::Space => self.world.get_mut::<Paused>().unwrap().toggle(),
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
