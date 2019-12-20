#![allow(clippy::type_complexity)]

extern crate ggez;
use ggez::*;

use std::collections::VecDeque;

extern crate specs;
use specs::prelude::*;

mod components;
use components::{
    Draw, Kinematics, Mass, Point, Position, Preview, Radius, SpeedGraph, Trail, Vector,
};

mod resources;
use resources::{
    MainIterations, NewPreview, Paused, PreviewIterations, Resolution, StartPoint, DT,
};

mod main_state;
use main_state::MainState;

mod physics_systems;
use physics_systems::{PhysicsSys, PreviewPhysicsSys};

mod trail_sys;
use trail_sys::{PreviewTrailSys, TrailSys};

mod graph_sys;
use graph_sys::SpeedGraphSys;

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
        Trail(VecDeque::with_capacity(36)),
    )
}

pub fn new_preview(pos: impl Into<Point>, vel: impl Into<Vector>, rad: f32) -> PreviewBody {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Radius(rad),
        Preview,
        Draw(graphics::Color::new(0.1, 1.0, 0.2, 0.8)),
        Trail(VecDeque::with_capacity(500)),
    )
}

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("N-body gravity sim", "Mikail Khan")
        .window_setup(ggez::conf::WindowSetup::default().title("Gravity"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(600.0, 600.0))
        .build()
        .expect("error building context");

    let mut world = World::new();

    world.register::<Position>();
    world.register::<Preview>();
    world.register::<Kinematics>();
    world.register::<Mass>();
    world.register::<Draw>();
    world.register::<Radius>();
    world.register::<Trail>();
    world.register::<SpeedGraph>();

    let data = vec![
        new_body([215.0, 100.0], [-0.0, -1.1], 0.01, 0.8),
        new_body([150.0, 100.0], [0.0, 0.0], 75.0, 5.0),
    ];

    // let data = (0..900)
    //     .map(|i| {
    //         new_body(
    //             [(i / 10) as f32 * 100.0, (i % 10) as f32 * 100.0],
    //             [0.0, 0.0],
    //             -0.1,
    //             5.0,
    //         )
    //     })
    //     .collect::<Vec<Body>>();

    for (pos, kine, mass, draw, rad, trail) in data {
        world
            .create_entity()
            .with(pos)
            .with(kine)
            .with(mass)
            .with(draw)
            .with(rad)
            .with(trail)
            .build();
    }

    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
    let dimensions = event_loop.get_primary_monitor().get_dimensions();
    let dimensions_vec = Vector::new(dimensions.width as f32, dimensions.height as f32);
    let aspect_ratio = dimensions.height / dimensions.width;

    world.insert(MainIterations(1));
    world.insert(PreviewIterations(25));
    world.insert(Resolution(dimensions_vec));
    world.insert(DT(1.0));
    world.insert(Paused(false));
    world.insert(StartPoint(None));
    world.insert(NewPreview(false));

    let mut main_dispatcher = DispatcherBuilder::new()
        .with(PhysicsSys, "physics_system", &[])
        .with(TrailSys, "trail_system", &["physics_system"])
        .with(SpeedGraphSys, "speed_graph_system", &["physics_system"])
        .build();

    let mut preview_dispatcher = DispatcherBuilder::new()
        .with(PreviewPhysicsSys, "preview_physics_system", &[])
        .with(
            PreviewTrailSys,
            "preview_trail_system",
            &["preview_physics_system"],
        )
        .build();

    main_dispatcher.setup(&mut world);
    preview_dispatcher.setup(&mut world);

    graphics::set_mode(
        ctx,
        ggez::conf::WindowMode::default()
            .resizable(true)
            .fullscreen_type(ggez::conf::FullscreenType::Desktop),
    )
    .expect("error resizing window");

    graphics::set_screen_coordinates(
        ctx,
        graphics::Rect::new(0., 0., SCREEN_X, SCREEN_Y * aspect_ratio as f32),
    )
    .unwrap();

    let main_state = &mut MainState::new(
        world,
        main_dispatcher,
        preview_dispatcher,
        ImGuiWrapper::new(ctx, hidpi_factor, dimensions_vec),
        hidpi_factor,
    );
    event::run(ctx, event_loop, main_state)
}
