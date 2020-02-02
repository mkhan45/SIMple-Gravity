use specs::prelude::*;

use ggez::{
    event::{self, EventHandler},
    graphics::{self, Color},
    input::{
        self,
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton,
    },
    Context, GameResult,
};

#[allow(unused_imports)]
use crate::ecs::{
    components::{
        AccelGraph, Draw, Kinematics, Mass, Position, Preview, Radius, SpeedGraph, Trail,
        XVelGraph, YVelGraph,
    },
    entities::{create_body, create_preview, new_body, new_preview},
    resources::{
        FollowSelectedBody, MousePos, NewPreview, Paused, RelativeTrails, Resolution, StartPoint,
    },
};

use crate::{Point, Vector};

use crate::gui::imgui_wrapper::*;
use crate::main_state::update_utils::calc_offset;

#[allow(unused_imports)]
use microprofile::scope;

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
            creating: false,
            items_hovered: false,
        }
    }
}

impl<'a, 'b> EventHandler for MainState<'a, 'b> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        microprofile::flip();
        microprofile::scope!("Update", "update");
        self.process_gui_signals();

        // unselect selected entity if it collided
        if let Some(e) = self.selected_entity {
            if !self.world.is_alive(e) {
                self.selected_entity = None;
                self.imgui_wrapper.sidemenu = false;
                self.world.insert(FollowSelectedBody(false));
            } else {
                let positions = self.world.read_storage::<Position>();
                let selected_pos = positions.get(e).expect("error getting selected position");
                self.world.fetch_mut::<RelativeTrails>().pos = Some(selected_pos.0);
            }
        }

        // apply camera movement
        let offset = calc_offset(ctx);
        if offset != [0.0, 0.0].into() {
            self.world.insert(FollowSelectedBody(false));
            let mut screen_coordinates = ggez::graphics::screen_coordinates(ctx);
            let zoom = screen_coordinates.w / crate::SCREEN_X;

            screen_coordinates.x += offset.x * zoom;
            screen_coordinates.y += offset.y * zoom;

            ggez::graphics::set_screen_coordinates(ctx, screen_coordinates).unwrap_or(());
        }

        if ggez::timer::ticks(ctx) % 120 == 0 {
            dbg!(ggez::timer::fps(ctx));
        }

        self.run_physics_systems(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        microprofile::scope!("Draw", "draw");
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        if self.world.fetch::<FollowSelectedBody>().0 {
            self.follow_selected_body(ctx)?;
        }
        self.update_gui_data();

        let mut builder = graphics::MeshBuilder::new();

        if self.world.fetch::<RelativeTrails>().enabled {
            let mut trail_builder = graphics::MeshBuilder::new();
            if let Some(dest_pos) = self.world.fetch::<RelativeTrails>().pos {
                self.draw_trails(&mut trail_builder);
                if let Ok(mesh) = trail_builder.build(ctx) {
                    graphics::draw(ctx, &mesh, graphics::DrawParam::new().dest(dest_pos))
                        .expect("error drawing mesh");
                }
            }
        } else {
            self.draw_trails(&mut builder);
        }

        self.draw_bodies(&mut builder);
        self.draw_preview(&mut builder, ctx);
        self.draw_vector_graphs(&mut builder, ctx);

        if let Some(ent) = self.selected_entity {
            self.draw_selected_outline(&mut builder, ent);
        }

        if let Ok(mesh) = builder.build(ctx) {
            graphics::draw(ctx, &mesh, graphics::DrawParam::new()).expect("error drawing mesh");
        }

        self.draw_gui(ctx);
        self.update_sim_data();

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
                    self.imgui_wrapper.remove_sidemenu();

                    if self.world.fetch::<RelativeTrails>().enabled {
                        (&mut self.world.write_storage::<Trail>())
                            .join()
                            .for_each(|trail| {
                                trail.points.clear();
                            });
                    }

                    let resolution = self.world.fetch::<Resolution>().0;
                    let screen_coordinates = graphics::screen_coordinates(ctx);
                    self.selected_entity = None;
                    self.imgui_wrapper.remove_sidemenu();
                    self.world.insert(FollowSelectedBody(false));
                    self.imgui_wrapper.render_data.entity_selected = false;

                    let positions = self.world.read_storage::<Position>();
                    let radii = self.world.read_storage::<Radius>();
                    let entities = self.world.entities();

                    let coords = ggez::graphics::screen_coordinates(ctx);
                    let mouse_pos = scale_pos([x, y], coords, resolution);

                    for (e, pos, rad) in (&entities, &positions, &radii).join() {
                        let mselect_rad = 12.5 * (screen_coordinates.w / resolution.x);
                        if pos.dist(mouse_pos) <= rad.0 + mselect_rad {
                            self.selected_entity = Some(e);
                            self.imgui_wrapper.render_data.entity_selected = true;
                            self.imgui_wrapper
                                .shown_menus
                                .insert(UiChoice::SideMenu(self.selected_entity));
                            break;
                        }
                    }
                }
                MouseButton::Left => {
                    // set up for creating new body
                    if self.creating {
                        let p = Point::new(x, y);
                        let coords = ggez::graphics::screen_coordinates(ctx);
                        let resolution = self.world.fetch::<Resolution>().0;
                        self.world
                            .insert(StartPoint(Some(scale_pos(p, coords, resolution))));

                        create_preview(
                            &mut self.world,
                            new_preview(p, [0.0, 0.0], self.imgui_wrapper.render_data.create_rad),
                        );
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

                        self.selected_entity = Some(create_body(
                            &mut self.world,
                            new_body(
                                start_point,
                                (start_point - p) * 0.025,
                                self.imgui_wrapper.render_data.create_mass,
                                self.imgui_wrapper.render_data.create_rad,
                            ),
                        ));
                        self.world.insert(StartPoint(None));
                    }
                }
                _ => dbg!(),
            }
        }

        self.delete_preview();
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
        self.world
            .insert(MousePos(input::mouse::position(ctx).into()));

        self.delete_preview();

        let mut coords = ggez::graphics::screen_coordinates(ctx);

        let start_point = self.world.fetch::<StartPoint>().0;
        if let Some(sp) = start_point {
            let resolution = self.world.fetch::<Resolution>().0;
            let p = scale_pos([x, y], coords, resolution);

            create_preview(
                &mut self.world,
                new_preview(
                    sp,
                    (sp - p) * 0.025,
                    self.imgui_wrapper.render_data.create_rad,
                ),
            );
        }

        if input::mouse::button_pressed(ctx, input::mouse::MouseButton::Middle) {
            self.world.insert(FollowSelectedBody(false));
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
        keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Space => self.world.get_mut::<Paused>().unwrap().toggle(),
            KeyCode::Escape => self.imgui_wrapper.remove_sidemenu(),
            KeyCode::F => self.world.insert(FollowSelectedBody(true)),
            KeyCode::R => self.imgui_wrapper.sent_signals.push(UiSignal::Delete),
            KeyCode::T => self.imgui_wrapper.sent_signals.push(UiSignal::ToggleTrails),
            KeyCode::G => self
                .imgui_wrapper
                .sent_signals
                .push(UiSignal::ToggleRelativeTrails),
            _ => {}
        };
        self.imgui_wrapper.update_key_down(keycode, keymods);
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

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.imgui_wrapper.update_key_up(keycode, keymods);
    }

    fn text_input_event(&mut self, _ctx: &mut Context, val: char) {
        self.imgui_wrapper.update_text(val);
    }
}
