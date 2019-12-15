extern crate ggez;
use ggez::*;

use std::collections::VecDeque;

extern crate specs;
use specs::prelude::*;

mod components;
use components::{Draw, Kinematics, Mass, Point, Position, Preview, Radius, Trail, Vector};

mod resources;
use resources::{MainIterations, PreviewIterations, Resolution, DT};

mod main_state;
use main_state::MainState;

mod imgui_wrapper;
// mod physics;
// mod trails;
use imgui_wrapper::ImGuiWrapper;

const G: f32 = 1.2;
const SCREEN_X: f32 = 300.0;
const SCREEN_Y: f32 = 300.0;

type Body = (Position, Kinematics, Mass, Draw, Radius, Trail);
type PreviewBody = (Position, Kinematics, Radius, Preview, Draw, Trail);

pub fn new_body(pos: impl Into<Point>, vel: impl Into<Vector>, mass: f32, rad: f32) -> Body {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Mass(mass),
        Draw(ggez::graphics::WHITE),
        Radius(rad),
        Trail(VecDeque::with_capacity(50)),
    )
}

pub fn new_preview(pos: impl Into<Point>, vel: impl Into<Vector>, rad: f32) -> PreviewBody {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Radius(rad),
        Preview,
        Draw(graphics::Color::new(0.1, 1.0, 0.2, 0.8)),
        Trail(VecDeque::with_capacity(50)),
    )
}

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("N-body gravity sim", "Mikail Khan")
        // .window_setup(ggez::conf::WindowSetup::default().title("Gravity"))
        // .window_mode(ggez::conf::WindowMode::default().dimensions(600.0, 600.0))
        .build()
        .expect("error building context");

    // let universe = Universe::new();
    // let mut world = universe.create_world();
    let mut world = World::new();
    // world.register::<Position>();
    // world.register::<Kinematics>();
    // world.register::<Mass>();
    // world.register::<Draw>();
    // world.register::<Radius>();
    // world.register::<Trail>();

    // let data = vec![
    //     new_body([215.0, 100.0], [-0.0, -1.1], 0.01, 0.8),
    //     new_body([150.0, 100.0], [0.0, 0.0], 75.0, 5.0),
    // ];

    // for (pos, kine, mass, draw, rad, trail) in data {
    //     world
    //         .create_entity()
    //         .with(pos)
    //         .with(kine)
    //         .with(mass)
    //         .with(draw)
    //         .with(rad)
    //         .with(trail)
    //         .build();
    // }

    // world.insert(
    //     (),
    //     (0..1000).map(|i| {
    //         (new_body(
    //             [(i / 10) as f32 * 100.0, (i % 10) as f32 * 100.0],
    //             [0.0, 0.0],
    //             -0.1,
    //             5.0,
    //         ))
    //     }),
    // );

    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
    let dimensions = event_loop.get_primary_monitor().get_dimensions();
    let dimensions_vec = Vector::new(dimensions.width as f32, dimensions.height as f32);
    let aspect_ratio = dimensions.height / dimensions.width;

    // world.insert(MainIterations(1));
    // world.insert(PreviewIterations(25));
    // world.insert(Resolution(dimensions_vec));
    // world.insert(DT(1.0));

    // graphics::set_mode(
    //     ctx,
    //     ggez::conf::WindowMode::default()
    //         .resizable(true)
    //         .fullscreen_type(ggez::conf::FullscreenType::Desktop),
    // )
    // .expect("error resizing window");

    // graphics::set_screen_coordinates(
    //     ctx,
    //     graphics::Rect::new(0., 0., SCREEN_X, SCREEN_Y * aspect_ratio as f32),
    // )
    // .unwrap();

    let main_state = &mut MainState::new(
        world,
        ImGuiWrapper::new(ctx, hidpi_factor, dimensions_vec),
        hidpi_factor,
    );
    event::run(ctx, event_loop, main_state)
}
